export interface AudioAsset {
  id: string;
  name: string;
  path: string;
  duration_secs: number;
  sample_rate: number;
  channels: number;
}

export type PlaybackMode = 'OneShot' | 'Loop' | 'Toggle';

export interface Pad {
  id: { '0': string };
  name: string;
  color: string;
  gain: number;
  is_muted: boolean;
  playback_mode: PlaybackMode;
  asset: AudioAsset | null;
  bank_index: number;
  slot_index: number;
}

export interface PadBank {
  index: number;
  name: string;
  pads: Pad[];
}

export interface ChannelStrip {
  gain: number;
  pan: number;
  is_muted: boolean;
  is_solo: boolean;
  peak_level: number;
}

export interface MixerState {
  mic_strip: ChannelStrip;
  pads_strip: ChannelStrip;
  music_strip: ChannelStrip;
  master_gain: number;
  master_muted?: boolean;
  master_peak_l: number;
  master_peak_r: number;
}

export interface Project {
  schema_version: number;
  app_version: string;
  name: string;
  banks: PadBank[];
  active_bank: number;
  mixer: MixerState;
  input_device_name: string | null;
  output_device_name: string | null;
  sample_rate: number;
  buffer_size: number;
}

export type RecordingState = 'idle' | 'armed' | 'recording' | 'paused';

export interface LevelSnapshot {
  master_l: number;
  master_r: number;
  mic: number;
}
