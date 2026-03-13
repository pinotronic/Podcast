use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PadId(pub String);

impl PadId {
    pub fn new() -> Self {
        PadId(Uuid::new_v4().to_string())
    }
}

impl Default for PadId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetId(pub String);

impl AssetId {
    pub fn new() -> Self {
        AssetId(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlaybackMode {
    OneShot,
    Loop,
    Toggle,
}

impl Default for PlaybackMode {
    fn default() -> Self {
        PlaybackMode::OneShot
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    pub id: AssetId,
    pub name: String,
    pub path: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pad {
    pub id: PadId,
    pub name: String,
    pub color: String,
    pub gain: f32,
    pub is_muted: bool,
    pub playback_mode: PlaybackMode,
    pub asset: Option<AudioAsset>,
    pub bank_index: usize,
    pub slot_index: usize,
}

impl Pad {
    pub fn new(slot_index: usize, bank_index: usize) -> Self {
        Pad {
            id: PadId::new(),
            name: format!("PAD {}", slot_index + 1),
            color: "#4a9eff".to_string(),
            gain: 1.0,
            is_muted: false,
            playback_mode: PlaybackMode::default(),
            asset: None,
            bank_index,
            slot_index,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.asset.is_some() && !self.is_muted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadBank {
    pub index: usize,
    pub name: String,
    pub pads: Vec<Pad>,
}

impl PadBank {
    pub fn new(index: usize, pad_count: usize) -> Self {
        PadBank {
            index,
            name: format!("Bank {}", index + 1),
            pads: (0..pad_count).map(|i| Pad::new(i, index)).collect(),
        }
    }
}
