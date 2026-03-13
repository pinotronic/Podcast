use anyhow::Result;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

use crate::audio_engine::decoder;
use crate::audio_engine::engine::{AudioEngine, EngineCommand};
use crate::domain::pad::{AssetId, AudioAsset};
use crate::domain::project::Project;
use crate::domain::session::EpisodeSession;

pub struct AppState {
    pub project: Mutex<Project>,
    pub session: Mutex<EpisodeSession>,
    pub engine: Mutex<AudioEngine>,
    /// asset_id -> (asset meta, decoded samples)
    pub asset_library: Mutex<HashMap<String, (AudioAsset, Arc<Vec<f32>>)>>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let project = Project::new("default");
        let session = EpisodeSession::new("Session 1", project.sample_rate);
        let state = AppState {
            project: Mutex::new(project),
            session: Mutex::new(session),
            engine: Mutex::new(AudioEngine::new()?),
            asset_library: Mutex::new(HashMap::new()),
        };
        state.reload_engine(None, None)?;
        Ok(state)
    }

    pub fn reload_engine(&self, input: Option<String>, output: Option<String>) -> Result<()> {
        let (master_gain, pads_gain, mic_gain, master_muted, pads_muted, mic_muted) = {
            let project = self.project.lock();
            (
                project.mixer.master_gain,
                project.mixer.pads_strip.gain,
                project.mixer.mic_strip.gain,
                project.mixer.master_muted,
                project.mixer.pads_strip.is_muted,
                project.mixer.mic_strip.is_muted,
            )
        };

        let engine = AudioEngine::new_with_devices(input.as_deref(), output.as_deref())?;
        engine.send(EngineCommand::SetMasterGain(master_gain));
        engine.send(EngineCommand::SetPadsGain(pads_gain));
        engine.send(EngineCommand::SetMicGain(mic_gain));
        engine.send(EngineCommand::SetMasterMuted(master_muted));
        engine.send(EngineCommand::SetPadsMuted(pads_muted));
        engine.send(EngineCommand::SetMicMuted(mic_muted));

        let sample_rate = engine.output_sample_rate();

        {
            let mut current_engine = self.engine.lock();
            *current_engine = engine;
        }

        {
            let mut project = self.project.lock();
            project.input_device_name = input.filter(|value| !value.is_empty());
            project.output_device_name = output.filter(|value| !value.is_empty());
            project.sample_rate = sample_rate;
        }

        {
            let mut session = self.session.lock();
            session.sample_rate = sample_rate;
        }

        Ok(())
    }

    pub fn activate_project(&self, project: Project) -> Result<Project> {
        let asset_library = Self::build_asset_library(&project);

        {
            let mut current_project = self.project.lock();
            *current_project = project;
        }

        {
            let mut library = self.asset_library.lock();
            *library = asset_library;
        }

        let (input, output) = {
            let current_project = self.project.lock();
            (
                current_project.input_device_name.clone(),
                current_project.output_device_name.clone(),
            )
        };

        self.reload_engine(input, output)?;

        let current_project = self.project.lock();
        Ok(current_project.clone())
    }

    fn build_asset_library(project: &Project) -> HashMap<String, (AudioAsset, Arc<Vec<f32>>)> {
        let mut assets = HashMap::new();

        for bank in &project.banks {
            for pad in &bank.pads {
                let Some(asset) = &pad.asset else {
                    continue;
                };

                if assets.contains_key(&asset.id.0) {
                    continue;
                }

                match decoder::decode_file(&asset.path) {
                    Ok(decoded) => {
                        let hydrated_asset = AudioAsset {
                            id: AssetId(asset.id.0.clone()),
                            name: asset.name.clone(),
                            path: asset.path.clone(),
                            duration_secs: decoded.duration_secs,
                            sample_rate: decoded.sample_rate,
                            channels: decoded.channels,
                        };
                        assets.insert(asset.id.0.clone(), (hydrated_asset, Arc::new(decoded.samples)));
                    }
                    Err(err) => {
                        warn!("Failed to hydrate asset {} from {}: {}", asset.name, asset.path, err);
                    }
                }
            }
        }

        assets
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::domain::pad::{AssetId, AudioAsset, PadBank};
    use crate::domain::project::Project;
    use crate::domain::mixer::MixerState;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_wav_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("podcast-console-asset-{}.wav", unique))
    }

    fn write_test_wav(path: &PathBuf) {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut writer = WavWriter::create(path, spec).expect("wav created");
        for _ in 0..128 {
            writer.write_sample(0.25_f32).expect("left sample written");
            writer.write_sample(-0.25_f32).expect("right sample written");
        }
        writer.finalize().expect("wav finalized");
    }

    #[test]
    fn build_asset_library_loads_samples_for_project_assets() {
        let wav_path = temp_wav_path();
        write_test_wav(&wav_path);

        let asset = AudioAsset {
            id: AssetId("asset-1".to_string()),
            name: "kick".to_string(),
            path: wav_path.to_string_lossy().to_string(),
            duration_secs: 0.0,
            sample_rate: 0,
            channels: 0,
        };

        let mut bank = PadBank::new(0, 1);
        bank.pads[0].asset = Some(asset);

        let project = Project {
            schema_version: 1,
            app_version: "0.1.0".to_string(),
            name: "rehydrate".to_string(),
            banks: vec![bank],
            active_bank: 0,
            mixer: MixerState::default(),
            input_device_name: None,
            output_device_name: None,
            sample_rate: 48_000,
            buffer_size: 512,
        };

        let library = AppState::build_asset_library(&project);
        let (loaded_asset, samples) = library.get("asset-1").expect("asset loaded into library");

        assert_eq!(loaded_asset.name, "kick");
        assert_eq!(loaded_asset.sample_rate, 48_000);
        assert_eq!(loaded_asset.channels, 2);
        assert!(!samples.is_empty());

        let _ = fs::remove_file(wav_path);
    }
}
