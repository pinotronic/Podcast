use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordingState {
    Idle,
    Armed,
    Recording,
    Paused,
}

impl Default for RecordingState {
    fn default() -> Self {
        RecordingState::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeSession {
    pub id: String,
    pub name: String,
    pub recording_state: RecordingState,
    pub duration_frames: u64,
    pub output_path: Option<String>,
    pub sample_rate: u32,
}

impl EpisodeSession {
    pub fn new(name: &str, sample_rate: u32) -> Self {
        EpisodeSession {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            recording_state: RecordingState::Idle,
            duration_frames: 0,
            output_path: None,
            sample_rate,
        }
    }
}
