import { create } from 'zustand';
import type { Project, LevelSnapshot, RecordingState } from '../types';
import { commands } from '../lib/tauri';

interface AppStore {
  project: Project | null;
  recordingState: RecordingState;
  recordingDuration: number;
  levels: LevelSnapshot;
  activePads: Set<string>;
  selectedPad: { bank: number; slot: number } | null;
  activePanel: 'pads' | 'mixer' | 'devices' | 'project';

  // Actions
  loadProject: () => Promise<void>;
  saveProject: () => Promise<void>;
  setActiveBank: (index: number) => Promise<void>;
  triggerPad: (bank: number, slot: number) => Promise<void>;
  stopPad: (bank: number, slot: number) => Promise<void>;
  stopAllPads: () => Promise<void>;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  refreshLevels: () => Promise<void>;
  setSelectedPad: (bank: number | null, slot: number | null) => void;
  setActivePanel: (panel: AppStore['activePanel']) => void;
  refreshRecordingDuration: () => Promise<void>;
}

export const useAppStore = create<AppStore>((set, get) => ({
  project: null,
  recordingState: 'idle',
  recordingDuration: 0,
  levels: { master_l: 0, master_r: 0, mic: 0 },
  activePads: new Set(),
  selectedPad: null,
  activePanel: 'pads',

  loadProject: async () => {
    const project = await commands.getProject();
    set({ project });
  },

  saveProject: async () => {
    await commands.saveProject();
  },

  setActiveBank: async (index) => {
    await commands.setActiveBank(index);
    set((s) => {
      if (!s.project) return s;
      return { project: { ...s.project, active_bank: index } };
    });
  },

  triggerPad: async (bank, slot) => {
    try {
      await commands.triggerPad(bank, slot);
      set((s) => {
        const next = new Set(s.activePads);
        next.add(`${bank}-${slot}`);
        return { activePads: next };
      });
      // Auto-remove after short time (for one-shot feedback)
      setTimeout(() => {
        set((s) => {
          const next = new Set(s.activePads);
          next.delete(`${bank}-${slot}`);
          return { activePads: next };
        });
      }, 300);
    } catch (e) {
      console.error('trigger_pad error:', e);
    }
  },

  stopPad: async (bank, slot) => {
    await commands.stopPad(bank, slot);
    set((s) => {
      const next = new Set(s.activePads);
      next.delete(`${bank}-${slot}`);
      return { activePads: next };
    });
  },

  stopAllPads: async () => {
    await commands.stopAllPads();
    set({ activePads: new Set() });
  },

  startRecording: async () => {
    await commands.startRecording();
    set({ recordingState: 'recording' });
  },

  stopRecording: async () => {
    await commands.stopRecording();
    set({ recordingState: 'idle', recordingDuration: 0 });
  },

  refreshLevels: async () => {
    try {
      const levels = await commands.getLevels();
      set({ levels });
    } catch { /* ignore */ }
  },

  refreshRecordingDuration: async () => {
    if (get().recordingState === 'recording') {
      try {
        const d = await commands.getRecordingDuration();
        set({ recordingDuration: d });
      } catch { /* ignore */ }
    }
  },

  setSelectedPad: (bank, slot) => {
    if (bank === null || slot === null) {
      set({ selectedPad: null });
    } else {
      set({ selectedPad: { bank, slot } });
    }
  },

  setActivePanel: (panel) => set({ activePanel: panel }),
}));
