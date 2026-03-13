import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { LevelSnapshot, Project } from '../types';

type SelectedDevices = { input: string | null; output: string | null };
type ImportAssetResult = {
  asset_id: string;
  name: string;
  duration_secs: number;
  sample_rate: number;
  channels: number;
};

const MOCK_PROJECT_LIST_KEY = 'podcast-console.mock.projects';
const MOCK_CURRENT_PROJECT_KEY = 'podcast-console.mock.current';
const MOCK_PROJECT_PREFIX = 'podcast-console.mock.project.';

const mockAssets = new Map<string, Project['banks'][number]['pads'][number]['asset']>();
const mockRecording = {
  state: 'idle' as 'idle' | 'recording',
  startedAt: 0,
  path: null as string | null,
};

function isTauriRuntime() {
  return typeof window !== 'undefined' && typeof (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== 'undefined';
}

function makeId(prefix: string) {
  return `${prefix}-${Math.random().toString(36).slice(2, 10)}`;
}

function cloneProject(project: Project): Project {
  return JSON.parse(JSON.stringify(project)) as Project;
}

function createMockProject(name = 'Browser Demo'): Project {
  return {
    schema_version: 1,
    app_version: '0.1.0-web',
    name,
    banks: Array.from({ length: 3 }, (_, bankIndex) => ({
      index: bankIndex,
      name: `Bank ${bankIndex + 1}`,
      pads: Array.from({ length: 12 }, (_, slotIndex) => ({
        id: { '0': makeId('pad') },
        name: `PAD ${slotIndex + 1}`,
        color: '#4a9eff',
        gain: 1,
        is_muted: false,
        playback_mode: 'OneShot',
        asset: null,
        bank_index: bankIndex,
        slot_index: slotIndex,
      })),
    })),
    active_bank: 0,
    mixer: {
      mic_strip: { gain: 1, pan: 0, is_muted: false, is_solo: false, peak_level: 0 },
      pads_strip: { gain: 1, pan: 0, is_muted: false, is_solo: false, peak_level: 0 },
      music_strip: { gain: 1, pan: 0, is_muted: false, is_solo: false, peak_level: 0 },
      master_gain: 1,
      master_muted: false,
      master_peak_l: 0,
      master_peak_r: 0,
    },
    input_device_name: 'Browser Microphone',
    output_device_name: 'Browser Default Output',
    sample_rate: 48000,
    buffer_size: 512,
  };
}

function getProjectStorageKey(name: string) {
  return `${MOCK_PROJECT_PREFIX}${name}`;
}

function getMockProjectNames() {
  const raw = localStorage.getItem(MOCK_PROJECT_LIST_KEY);
  if (!raw) return [] as string[];
  try {
    return JSON.parse(raw) as string[];
  } catch {
    return [];
  }
}

function setMockProjectNames(names: string[]) {
  localStorage.setItem(MOCK_PROJECT_LIST_KEY, JSON.stringify([...new Set(names)]));
}

function persistMockProject(project: Project) {
  localStorage.setItem(getProjectStorageKey(project.name), JSON.stringify(project));
  localStorage.setItem(MOCK_CURRENT_PROJECT_KEY, project.name);
  setMockProjectNames([...getMockProjectNames(), project.name]);
}

function getMockProject(name?: string): Project {
  const currentName = name ?? localStorage.getItem(MOCK_CURRENT_PROJECT_KEY) ?? 'Browser Demo';
  const raw = localStorage.getItem(getProjectStorageKey(currentName));
  if (!raw) {
    const project = createMockProject(currentName);
    persistMockProject(project);
    return project;
  }
  try {
    return JSON.parse(raw) as Project;
  } catch {
    const project = createMockProject(currentName);
    persistMockProject(project);
    return project;
  }
}

function updateMockProject(mutator: (project: Project) => void) {
  const project = getMockProject();
  mutator(project);
  persistMockProject(project);
  return project;
}

function getAssetName(path: string) {
  const fileName = path.split(/[\\/]/).pop() ?? path;
  return fileName.replace(/\.[^.]+$/, '') || 'asset';
}

function buildMockLevels(): LevelSnapshot {
  const project = getMockProject();
  const activity = mockRecording.state === 'recording' ? 0.18 : 0.06;
  const jitter = Math.random() * 0.08;
  const master = Math.min(1, project.mixer.master_gain * (activity + jitter));
  const mic = project.mixer.mic_strip.is_muted ? 0 : Math.min(1, project.mixer.mic_strip.gain * activity * 0.5);
  return {
    master_l: master,
    master_r: Math.max(0, master - 0.03),
    mic,
  };
}

async function openBrowserAudioFile(): Promise<string | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'audio/*';
    input.onchange = () => {
      const file = input.files?.[0];
      resolve(file ? `browser://${file.name}` : null);
    };
    input.click();
  });
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

export const commands = {
  getProject: async () => (isTauriRuntime() ? call<Project>('get_project') : cloneProject(getMockProject())),
  saveProject: async () => {
    if (isTauriRuntime()) {
      await call<void>('save_project');
      return;
    }
    persistMockProject(getMockProject());
  },
  createProject: async (name: string) => {
    if (isTauriRuntime()) return call<Project>('create_project', { name });
    const project = createMockProject(name);
    persistMockProject(project);
    return cloneProject(project);
  },
  listProjects: async () => (isTauriRuntime() ? call<string[]>('list_projects') : getMockProjectNames()),
  openProject: async (name: string) => {
    if (isTauriRuntime()) return call<Project>('open_project', { name });
    const project = getMockProject(name);
    persistMockProject(project);
    return cloneProject(project);
  },
  setActiveBank: async (index: number) => {
    if (isTauriRuntime()) {
      await call<void>('set_active_bank', { index });
      return;
    }
    updateMockProject((project) => {
      project.active_bank = index;
    });
  },
  importAsset: async (path: string) => {
    if (isTauriRuntime()) return call<ImportAssetResult>('import_asset', { path });
    const asset = {
      id: makeId('asset'),
      name: getAssetName(path),
      path,
      duration_secs: 3.5,
      sample_rate: 48000,
      channels: 2,
    };
    mockAssets.set(asset.id, asset);
    return {
      asset_id: asset.id,
      name: asset.name,
      duration_secs: asset.duration_secs,
      sample_rate: asset.sample_rate,
      channels: asset.channels,
    };
  },
  assignAssetToPad: async (bankIndex: number, slotIndex: number, assetId: string) => {
    if (isTauriRuntime()) {
      await call<void>('assign_asset_to_pad', { bankIndex, slotIndex, assetId });
      return;
    }
    updateMockProject((project) => {
      const pad = project.banks[bankIndex]?.pads[slotIndex];
      if (!pad) return;
      pad.asset = mockAssets.get(assetId) ?? null;
    });
  },
  triggerPad: async (bankIndex: number, slotIndex: number) => {
    if (isTauriRuntime()) return call<string>('trigger_pad', { bankIndex, slotIndex });
    return `${bankIndex}-${slotIndex}-${Date.now()}`;
  },
  stopPad: async (bankIndex: number, slotIndex: number) => {
    if (isTauriRuntime()) {
      await call<void>('stop_pad', { bankIndex, slotIndex });
      return;
    }
  },
  stopAllPads: async () => {
    if (isTauriRuntime()) {
      await call<void>('stop_all_pads');
    }
  },
  updatePadName: async (bankIndex: number, slotIndex: number, name: string) => {
    if (isTauriRuntime()) {
      await call<void>('update_pad_name', { bankIndex, slotIndex, name });
      return;
    }
    updateMockProject((project) => {
      const pad = project.banks[bankIndex]?.pads[slotIndex];
      if (pad) pad.name = name;
    });
  },
  updatePadColor: async (bankIndex: number, slotIndex: number, color: string) => {
    if (isTauriRuntime()) {
      await call<void>('update_pad_color', { bankIndex, slotIndex, color });
      return;
    }
    updateMockProject((project) => {
      const pad = project.banks[bankIndex]?.pads[slotIndex];
      if (pad) pad.color = color;
    });
  },
  updatePadGain: async (bankIndex: number, slotIndex: number, gain: number) => {
    if (isTauriRuntime()) {
      await call<void>('update_pad_gain', { bankIndex, slotIndex, gain });
      return;
    }
    updateMockProject((project) => {
      const pad = project.banks[bankIndex]?.pads[slotIndex];
      if (pad) pad.gain = Math.max(0, Math.min(2, gain));
    });
  },
  setPadPlaybackMode: async (bankIndex: number, slotIndex: number, mode: string) => {
    if (isTauriRuntime()) {
      await call<void>('set_pad_playback_mode', { bankIndex, slotIndex, mode });
      return;
    }
    updateMockProject((project) => {
      const pad = project.banks[bankIndex]?.pads[slotIndex];
      if (!pad) return;
      pad.playback_mode = mode === 'loop' ? 'Loop' : mode === 'toggle' ? 'Toggle' : 'OneShot';
    });
  },
  switchBank: async (bankIndex: number) => {
    if (isTauriRuntime()) {
      await call<void>('switch_bank', { bankIndex });
      return;
    }
    updateMockProject((project) => {
      project.active_bank = bankIndex;
    });
  },
  startRecording: async () => {
    if (isTauriRuntime()) return call<string>('start_recording');
    mockRecording.state = 'recording';
    mockRecording.startedAt = Date.now();
    mockRecording.path = `browser-recordings/${getMockProject().name}-${mockRecording.startedAt}.wav`;
    return mockRecording.path;
  },
  stopRecording: async () => {
    if (isTauriRuntime()) return call<string | null>('stop_recording');
    mockRecording.state = 'idle';
    return mockRecording.path;
  },
  getRecordingState: async () => (isTauriRuntime() ? call<string>('get_recording_state') : mockRecording.state),
  getRecordingDuration: async () => {
    if (isTauriRuntime()) return call<number>('get_recording_duration');
    if (mockRecording.state !== 'recording') return 0;
    return (Date.now() - mockRecording.startedAt) / 1000;
  },
  setMasterGain: async (gain: number) => {
    if (isTauriRuntime()) {
      await call<void>('set_master_gain', { gain });
      return;
    }
    updateMockProject((project) => {
      project.mixer.master_gain = Math.max(0, Math.min(2, gain));
    });
  },
  setPadsGain: async (gain: number) => {
    if (isTauriRuntime()) {
      await call<void>('set_pads_gain', { gain });
      return;
    }
    updateMockProject((project) => {
      project.mixer.pads_strip.gain = Math.max(0, Math.min(2, gain));
    });
  },
  setMicGain: async (gain: number) => {
    if (isTauriRuntime()) {
      await call<void>('set_mic_gain', { gain });
      return;
    }
    updateMockProject((project) => {
      project.mixer.mic_strip.gain = Math.max(0, Math.min(2, gain));
    });
  },
  getLevels: async () => (isTauriRuntime() ? call<LevelSnapshot>('get_levels') : buildMockLevels()),
  toggleMuteStrip: async (strip: string) => {
    if (isTauriRuntime()) return call<boolean>('toggle_mute_strip', { strip });
    let muted = false;
    updateMockProject((project) => {
      if (strip === 'mic') {
        project.mixer.mic_strip.is_muted = !project.mixer.mic_strip.is_muted;
        muted = project.mixer.mic_strip.is_muted;
      } else if (strip === 'pads') {
        project.mixer.pads_strip.is_muted = !project.mixer.pads_strip.is_muted;
        muted = project.mixer.pads_strip.is_muted;
      } else if (strip === 'master') {
        project.mixer.master_muted = !project.mixer.master_muted;
        muted = project.mixer.master_muted;
      }
    });
    return muted;
  },
  listInputDevices: async () => (isTauriRuntime() ? call<string[]>('list_input_devices') : ['Browser Microphone']),
  listOutputDevices: async () => (isTauriRuntime() ? call<string[]>('list_output_devices') : ['Browser Default Output']),
  getSelectedDevices: async () => (isTauriRuntime() ? call<SelectedDevices>('get_selected_devices') : {
    input: getMockProject().input_device_name,
    output: getMockProject().output_device_name,
  }),
  setSelectedDevices: async (input: string | null, output: string | null) => {
    if (isTauriRuntime()) {
      return call<SelectedDevices>('set_selected_devices', { input, output });
    }
    updateMockProject((project) => {
      project.input_device_name = input;
      project.output_device_name = output;
    });
    return { input, output };
  },
};

export async function pickAudioFile(): Promise<string | null> {
  if (isTauriRuntime()) {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg'] }],
    });
    if (!selected || Array.isArray(selected)) return null;
    return selected;
  }

  return openBrowserAudioFile();
}
