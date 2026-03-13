use anyhow::Result;
use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{error, info};

use super::mixer;
use super::recorder::Recorder;
use super::scheduler::{PadVoice, Scheduler};

pub enum EngineCommand {
    TriggerPad(PadVoice),
    StopPad(String),
    StopAll,
    StartRecording(String, u32, u16),
    StopRecording,
    SetMasterGain(f32),
    SetPadsGain(f32),
    SetMicGain(f32),
    SetMasterMuted(bool),
    SetPadsMuted(bool),
    SetMicMuted(bool),
}

pub struct LevelSnapshot {
    pub master_l: f32,
    pub master_r: f32,
    pub mic: f32,
}

/// All Send+Sync: only channels, no cpal::Stream.
pub struct AudioEngine {
    cmd_tx: Sender<EngineCommand>,
    pub level_rx: Arc<Mutex<Receiver<LevelSnapshot>>>,
    recorder: Arc<Mutex<Recorder>>,
    output_sample_rate: u32,
    output_channels: u16,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        Self::new_with_devices(None, None)
    }

    pub fn new_with_devices(input_device_name: Option<&str>, output_device_name: Option<&str>) -> Result<Self> {
        let host = cpal::default_host();
        let selected_input_name = input_device_name.map(str::to_owned);

        let output_device = match output_device_name {
            Some(name) if !name.is_empty() => Self::find_output_device_by_name(&host, name)
                .ok_or_else(|| anyhow::anyhow!("Output device not found: {}", name))?,
            _ => host
                .default_output_device()
                .ok_or_else(|| anyhow::anyhow!("No output device"))?,
        };
        let output_config: StreamConfig = output_device.default_output_config()?.into();
        let sample_rate = output_config.sample_rate.0;
        let out_channels = output_config.channels as usize;

        info!(
            "Audio: output={} sr={} ch={}",
            output_device.name().unwrap_or_default(),
            sample_rate,
            out_channels
        );

        let scheduler = Scheduler::new();
        let recorder = Recorder::new();

        let (cmd_tx, cmd_rx) = bounded::<EngineCommand>(256);
        let (level_tx, level_rx) = bounded::<LevelSnapshot>(64);

        let master_gain = Arc::new(Mutex::new(1.0f32));
        let pads_gain = Arc::new(Mutex::new(1.0f32));
    let mic_gain = Arc::new(Mutex::new(1.0f32));
        let master_muted = Arc::new(Mutex::new(false));
        let pads_muted = Arc::new(Mutex::new(false));
    let mic_muted = Arc::new(Mutex::new(false));
    let mic_peak = Arc::new(Mutex::new(0.0f32));
    let mic_buffer = Arc::new(Mutex::new(VecDeque::<f32>::new()));

        let master_gain_cb = master_gain.clone();
        let pads_gain_cb = pads_gain.clone();
    let mic_gain_cb = mic_gain.clone();
        let master_muted_cb = master_muted.clone();
        let pads_muted_cb = pads_muted.clone();
    let mic_muted_cb = mic_muted.clone();
    let mic_peak_out = mic_peak.clone();
    let mic_buffer_out = mic_buffer.clone();
        let scheduler_cb = scheduler.clone();
        let recorder_cb = recorder.clone();
        let recorder_thread = recorder.clone();

        // Spawn a thread that owns the cpal stream its entire lifetime
        std::thread::spawn(move || {
            let stream = output_device
                .build_output_stream(
                    &output_config,
                    move |data: &mut [f32], _| {
                        for s in data.iter_mut() { *s = 0.0; }

                        let mic_level = {
                            let muted = *mic_muted_cb.lock();
                            let gain = if muted { 0.0 } else { *mic_gain_cb.lock() };
                            if gain > 0.0 {
                                Self::mix_input_buffer(&mic_buffer_out, data, gain);
                            }

                            let mut peak = mic_peak_out.lock();
                            let level = if muted { 0.0 } else { (*peak * gain).min(1.0) };
                            *peak = 0.0;
                            level
                        };

                        let pg = if *pads_muted_cb.lock() {
                            0.0
                        } else {
                            *pads_gain_cb.lock()
                        };
                        scheduler_cb.process(data, out_channels, pg);

                        let mg = if *master_muted_cb.lock() {
                            0.0
                        } else {
                            *master_gain_cb.lock()
                        };
                        mixer::apply_gain(data, mg);
                        mixer::limit(data, 1.0);

                        let peak_l = mixer::peak(
                            &data.iter().step_by(out_channels).cloned().collect::<Vec<_>>()
                        );
                        let peak_r = if out_channels > 1 {
                            mixer::peak(&data.iter().skip(1).step_by(out_channels).cloned().collect::<Vec<_>>())
                        } else { peak_l };

                        let _ = level_tx.try_send(LevelSnapshot { master_l: peak_l, master_r: peak_r, mic: mic_level });

                        let mut rec = recorder_cb.lock();
                        if rec.is_recording() {
                            let _ = rec.write(data);
                        }
                    },
                    |err| error!("Output error: {}", err),
                    None,
                )
                .expect("Failed to build output stream");

            stream.play().expect("Failed to play stream");

            // Try to open input
            let input_device = match selected_input_name.as_deref() {
                Some(name) if !name.is_empty() => match Self::find_input_device_by_name(&host, name) {
                    Some(device) => Some(device),
                    None => {
                        error!("Input device not found: {}", name);
                        None
                    }
                },
                _ => host.default_input_device(),
            };

            if let Some(input_device) = input_device {
                if let Ok(input_config) = input_device.default_input_config() {
                    let ic: StreamConfig = input_config.config();
                    let input_channels = ic.channels as usize;
                    let output_channels_input = out_channels;
                    let mic_peak_in = mic_peak.clone();
                    let mic_buffer_in = mic_buffer.clone();
                    let build_result = match input_config.sample_format() {
                        SampleFormat::F32 => input_device.build_input_stream(
                            &ic,
                            move |data: &[f32], _| {
                                Self::capture_input_buffer(
                                    data,
                                    input_channels,
                                    input_config.sample_rate().0,
                                    output_channels_input,
                                    sample_rate,
                                    &mic_buffer_in,
                                    &mic_peak_in,
                                );
                            },
                            |e| error!("Input error: {}", e),
                            None,
                        ),
                        SampleFormat::I16 => input_device.build_input_stream(
                            &ic,
                            move |data: &[i16], _| {
                                let normalized = data
                                    .iter()
                                    .map(|sample| *sample as f32 / i16::MAX as f32)
                                    .collect::<Vec<_>>();
                                Self::capture_input_buffer(
                                    &normalized,
                                    input_channels,
                                    input_config.sample_rate().0,
                                    output_channels_input,
                                    sample_rate,
                                    &mic_buffer_in,
                                    &mic_peak_in,
                                );
                            },
                            |e| error!("Input error: {}", e),
                            None,
                        ),
                        SampleFormat::U16 => input_device.build_input_stream(
                            &ic,
                            move |data: &[u16], _| {
                                let normalized = data
                                    .iter()
                                    .map(|sample| (*sample as f32 / u16::MAX as f32) * 2.0 - 1.0)
                                    .collect::<Vec<_>>();
                                Self::capture_input_buffer(
                                    &normalized,
                                    input_channels,
                                    input_config.sample_rate().0,
                                    output_channels_input,
                                    sample_rate,
                                    &mic_buffer_in,
                                    &mic_peak_in,
                                );
                            },
                            |e| error!("Input error: {}", e),
                            None,
                        ),
                        sample_format => {
                            error!("Unsupported input sample format: {:?}", sample_format);
                            Err(cpal::BuildStreamError::StreamConfigNotSupported)
                        }
                    };

                    if let Ok(istream) = build_result {
                        let _ = istream.play();
                        // Process commands while keeping both streams alive
                        while let Ok(cmd) = cmd_rx.recv() {
                            Self::handle_cmd(cmd, &scheduler, &recorder_thread, &master_gain, &pads_gain, &mic_gain, &master_muted, &pads_muted, &mic_muted);
                        }
                        drop(istream);
                    } else {
                        while let Ok(cmd) = cmd_rx.recv() {
                            Self::handle_cmd(cmd, &scheduler, &recorder_thread, &master_gain, &pads_gain, &mic_gain, &master_muted, &pads_muted, &mic_muted);
                        }
                    }
                } else {
                    while let Ok(cmd) = cmd_rx.recv() {
                        Self::handle_cmd(cmd, &scheduler, &recorder_thread, &master_gain, &pads_gain, &mic_gain, &master_muted, &pads_muted, &mic_muted);
                    }
                }
            } else {
                while let Ok(cmd) = cmd_rx.recv() {
                    Self::handle_cmd(cmd, &scheduler, &recorder_thread, &master_gain, &pads_gain, &mic_gain, &master_muted, &pads_muted, &mic_muted);
                }
            }

            drop(stream);
        });

        Ok(AudioEngine {
            cmd_tx,
            level_rx: Arc::new(Mutex::new(level_rx)),
            recorder,
            output_sample_rate: sample_rate,
            output_channels: out_channels as u16,
        })
    }

    fn capture_input_buffer(
        input: &[f32],
        input_channels: usize,
        input_sample_rate: u32,
        output_channels: usize,
        output_sample_rate: u32,
        mic_buffer: &Arc<Mutex<VecDeque<f32>>>,
        mic_peak: &Arc<Mutex<f32>>,
    ) {
        if input.is_empty() || input_channels == 0 || output_channels == 0 {
            return;
        }

        let peak = mixer::peak(input);
        {
            let mut current_peak = mic_peak.lock();
            if peak > *current_peak {
                *current_peak = peak;
            }
        }

        let input_frames = input.len() / input_channels;
        if input_frames == 0 {
            return;
        }

        let ratio = output_sample_rate as f64 / input_sample_rate.max(1) as f64;
        let output_frames = ((input_frames as f64) * ratio).round().max(1.0) as usize;
        let max_buffer_len = output_sample_rate as usize * output_channels * 2;

        let mut queue = mic_buffer.lock();
        for out_frame in 0..output_frames {
            let src_frame = ((out_frame as f64) / ratio).floor() as usize;
            let frame_index = src_frame.min(input_frames - 1);
            for channel in 0..output_channels {
                let src_channel = if input_channels == 1 {
                    0
                } else {
                    channel.min(input_channels - 1)
                };
                let src_index = frame_index * input_channels + src_channel;
                queue.push_back(input[src_index]);
            }
        }

        while queue.len() > max_buffer_len {
            queue.pop_front();
        }
    }

    fn mix_input_buffer(mic_buffer: &Arc<Mutex<VecDeque<f32>>>, output: &mut [f32], gain: f32) {
        let mut queue = mic_buffer.lock();
        for sample in output.iter_mut() {
            if let Some(mic_sample) = queue.pop_front() {
                *sample += mic_sample * gain;
            } else {
                break;
            }
        }
    }

    fn handle_cmd(
        cmd: EngineCommand,
        scheduler: &Arc<Scheduler>,
        recorder: &Arc<Mutex<super::recorder::Recorder>>,
        master_gain: &Arc<Mutex<f32>>,
        pads_gain: &Arc<Mutex<f32>>,
        mic_gain: &Arc<Mutex<f32>>,
        master_muted: &Arc<Mutex<bool>>,
        pads_muted: &Arc<Mutex<bool>>,
        mic_muted: &Arc<Mutex<bool>>,
    ) {
        match cmd {
            EngineCommand::TriggerPad(voice) => { scheduler.trigger(voice); }
            EngineCommand::StopPad(id) => { scheduler.stop_pad(&id); }
            EngineCommand::StopAll => { scheduler.stop_all(); }
            EngineCommand::StartRecording(path, sr, ch) => {
                let mut rec = recorder.lock();
                if let Err(e) = rec.start(&path, sr, ch) {
                    error!("Recording start: {}", e);
                }
            }
            EngineCommand::StopRecording => {
                let mut rec = recorder.lock();
                let _ = rec.stop();
            }
            EngineCommand::SetMasterGain(g) => { *master_gain.lock() = g; }
            EngineCommand::SetPadsGain(g) => { *pads_gain.lock() = g; }
            EngineCommand::SetMicGain(g) => { *mic_gain.lock() = g; }
            EngineCommand::SetMasterMuted(muted) => { *master_muted.lock() = muted; }
            EngineCommand::SetPadsMuted(muted) => { *pads_muted.lock() = muted; }
            EngineCommand::SetMicMuted(muted) => { *mic_muted.lock() = muted; }
        }
    }

    pub fn send(&self, cmd: EngineCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    pub fn list_output_devices() -> Vec<String> {
        cpal::default_host()
            .output_devices()
            .map(|d| d.filter_map(|dev| dev.name().ok()).collect())
            .unwrap_or_default()
    }

    pub fn list_input_devices() -> Vec<String> {
        cpal::default_host()
            .input_devices()
            .map(|d| d.filter_map(|dev| dev.name().ok()).collect())
            .unwrap_or_default()
    }

    fn find_output_device_by_name(host: &cpal::Host, name: &str) -> Option<cpal::Device> {
        host.output_devices()
            .ok()?
            .find(|device| device.name().ok().as_deref() == Some(name))
    }

    fn find_input_device_by_name(host: &cpal::Host, name: &str) -> Option<cpal::Device> {
        host.input_devices()
            .ok()?
            .find(|device| device.name().ok().as_deref() == Some(name))
    }

    pub fn recording_duration_secs(&self) -> f64 {
        let recorder = self.recorder.lock();
        if recorder.sample_rate == 0 {
            return 0.0;
        }
        recorder.frames_written as f64 / recorder.sample_rate as f64
    }

    pub fn output_sample_rate(&self) -> u32 {
        self.output_sample_rate
    }

    pub fn output_channels(&self) -> u16 {
        self.output_channels
    }
}

// Safe because AudioEngine only holds Sender (Send) and Arc<Mutex<Receiver>> (Send+Sync)
unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}
