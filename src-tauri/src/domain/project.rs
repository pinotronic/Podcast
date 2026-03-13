use serde::{Deserialize, Serialize};
use crate::domain::pad::PadBank;
use crate::domain::mixer::MixerState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub schema_version: u32,
    pub app_version: String,
    pub name: String,
    pub banks: Vec<PadBank>,
    pub active_bank: usize,
    pub mixer: MixerState,
    pub input_device_name: Option<String>,
    pub output_device_name: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: u32,
}

impl Project {
    pub fn new(name: &str) -> Self {
        let banks = vec![
            PadBank::new(0, 12),
            PadBank::new(1, 12),
            PadBank::new(2, 12),
        ];
        Project {
            schema_version: 1,
            app_version: "0.1.0".to_string(),
            name: name.to_string(),
            banks,
            active_bank: 0,
            mixer: MixerState::default(),
            input_device_name: None,
            output_device_name: None,
            sample_rate: 48000,
            buffer_size: 512,
        }
    }
}
