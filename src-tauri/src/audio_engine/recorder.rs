use anyhow::Result;
use hound::{WavSpec, WavWriter, SampleFormat};
use std::sync::Arc;
use parking_lot::Mutex;

pub struct Recorder {
    writer: Option<WavWriter<std::io::BufWriter<std::fs::File>>>,
    pub path: Option<String>,
    pub frames_written: u64,
    pub sample_rate: u32,
    pub channels: u16,
}

impl Recorder {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Recorder {
            writer: None,
            path: None,
            frames_written: 0,
            sample_rate: 48000,
            channels: 2,
        }))
    }

    pub fn start(&mut self, path: &str, sample_rate: u32, channels: u16) -> Result<()> {
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let writer = WavWriter::create(path, spec)?;
        self.writer = Some(writer);
        self.path = Some(path.to_string());
        self.frames_written = 0;
        self.sample_rate = sample_rate;
        self.channels = channels;
        Ok(())
    }

    pub fn write(&mut self, buf: &[f32]) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            for &sample in buf {
                writer.write_sample(sample)?;
            }
            self.frames_written += buf.len() as u64 / self.channels as u64;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Option<String>> {
        if let Some(writer) = self.writer.take() {
            writer.finalize()?;
        }
        Ok(self.path.clone())
    }

    pub fn is_recording(&self) -> bool {
        self.writer.is_some()
    }
}
