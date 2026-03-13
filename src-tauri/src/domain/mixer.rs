use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStrip {
    pub gain: f32,
    pub pan: f32,
    pub is_muted: bool,
    pub is_solo: bool,
    pub peak_level: f32,
}

impl Default for ChannelStrip {
    fn default() -> Self {
        ChannelStrip {
            gain: 1.0,
            pan: 0.0,
            is_muted: false,
            is_solo: false,
            peak_level: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MixerState {
    pub mic_strip: ChannelStrip,
    pub pads_strip: ChannelStrip,
    pub music_strip: ChannelStrip,
    pub master_gain: f32,
    pub master_muted: bool,
    pub master_peak_l: f32,
    pub master_peak_r: f32,
}

impl Default for MixerState {
    fn default() -> Self {
        MixerState {
            mic_strip: ChannelStrip::default(),
            pads_strip: ChannelStrip::default(),
            music_strip: ChannelStrip::default(),
            master_gain: 1.0,
            master_muted: false,
            master_peak_l: 0.0,
            master_peak_r: 0.0,
        }
    }
}
