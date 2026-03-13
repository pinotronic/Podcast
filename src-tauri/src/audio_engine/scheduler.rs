use std::sync::Arc;
use parking_lot::Mutex;

/// A playing pad voice: decoded PCM + cursor position.
pub struct PadVoice {
    pub pad_id: String,
    pub playback_id: String,
    pub samples: Arc<Vec<f32>>,
    pub channels: u16,
    pub gain: f32,
    pub cursor: usize,
    pub looping: bool,
    pub finished: bool,
}

impl PadVoice {
    pub fn read_into(&mut self, out: &mut [f32], out_channels: usize) {
        let total = self.samples.len();
        if total == 0 || self.finished {
            return;
        }
        let out_frames = out.len() / out_channels;
        for frame in 0..out_frames {
            if self.cursor >= total {
                if self.looping {
                    self.cursor = 0;
                } else {
                    self.finished = true;
                    break;
                }
            }
            let src_ch = self.channels as usize;
            for c in 0..out_channels {
                let src_idx = if src_ch == 1 {
                    self.cursor
                } else {
                    self.cursor + (c % src_ch)
                };
                let s = if src_idx < total { self.samples[src_idx] } else { 0.0 };
                out[frame * out_channels + c] += s * self.gain;
            }
            self.cursor += src_ch;
        }
    }
}

pub struct Scheduler {
    pub voices: Mutex<Vec<PadVoice>>,
}

impl Scheduler {
    pub fn new() -> Arc<Self> {
        Arc::new(Scheduler {
            voices: Mutex::new(Vec::new()),
        })
    }

    pub fn trigger(&self, voice: PadVoice) {
        let mut voices = self.voices.lock();
        // Remove existing voice for same pad (one-shot replaces)
        voices.retain(|v| v.pad_id != voice.pad_id);
        voices.push(voice);
    }

    pub fn stop_pad(&self, pad_id: &str) {
        let mut voices = self.voices.lock();
        voices.retain(|v| v.pad_id != pad_id);
    }

    pub fn stop_all(&self) {
        let mut voices = self.voices.lock();
        voices.clear();
    }

    /// Mix all voices into output buffer. Returns list of finished playback_ids.
    pub fn process(&self, out: &mut [f32], out_channels: usize, gain: f32) -> Vec<(String, String)> {
        let mut finished = Vec::new();
        let mut voices = self.voices.lock();
        for voice in voices.iter_mut() {
            voice.read_into(out, out_channels);
            if voice.finished {
                finished.push((voice.pad_id.clone(), voice.playback_id.clone()));
            }
        }
        // Apply bus gain
        for s in out.iter_mut() {
            *s *= gain;
        }
        voices.retain(|v| !v.finished);
        finished
    }
}

#[cfg(test)]
mod tests {
    use super::PadVoice;
    use std::sync::Arc;

    #[test]
    fn mono_voice_is_duplicated_to_stereo_output() {
        let mut voice = PadVoice {
            pad_id: "pad-1".to_string(),
            playback_id: "playback-1".to_string(),
            samples: Arc::new(vec![0.5, -0.25]),
            channels: 1,
            gain: 1.0,
            cursor: 0,
            looping: false,
            finished: false,
        };

        let mut out = vec![0.0; 4];
        voice.read_into(&mut out, 2);

        assert_eq!(out, vec![0.5, 0.5, -0.25, -0.25]);
    }
}
