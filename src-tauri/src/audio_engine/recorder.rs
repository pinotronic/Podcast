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

#[cfg(test)]
mod tests {
    use super::Recorder;
    use hound::WavReader;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_wav_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("podcast-console-recorder-{}.wav", unique))
    }

    #[test]
    fn recorder_writes_valid_wav_file() {
        let path = temp_wav_path();
        let recorder = Recorder::new();

        {
            let mut recorder = recorder.lock();
            recorder.start(path.to_str().expect("utf8 path"), 48_000, 2).expect("recording started");
            recorder
                .write(&[0.25, -0.25, 0.5, -0.5, 0.125, -0.125, 0.0, 0.0])
                .expect("samples written");
            recorder.stop().expect("recording stopped");
        }

        let reader = WavReader::open(&path).expect("wav readable");
        let spec = reader.spec();
        let samples = reader
            .into_samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .expect("samples decoded");

        assert_eq!(spec.sample_rate, 48_000);
        assert_eq!(spec.channels, 2);
        assert_eq!(samples.len(), 8);

        let _ = fs::remove_file(path);
    }
}
