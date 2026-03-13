import { create } from 'zustand';
import type { Project, LevelSnapshot, RecordingState } from '../types';
import { commands } from '../lib/tauri';

type NoticeKind = 'error' | 'success' | 'info';

interface AppStore {
  project: Project | null;
  recordingState: RecordingState;
  recordingDuration: number;
  levels: LevelSnapshot;
  activePads: Set<string>;
  selectedPad: { bank: number; slot: number } | null;
  activePanel: 'pads' | 'mixer' | 'devices' | 'project';
  notice: { kind: NoticeKind; message: string } | null;

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
  setNotice: (kind: NoticeKind, message: string) => void;
  clearNotice: () => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  project: null,
  recordingState: 'idle',
  recordingDuration: 0,
  levels: { master_l: 0, master_r: 0, mic: 0 },
  activePads: new Set(),
  selectedPad: null,
  activePanel: 'pads',
  notice: null,

  loadProject: async () => {
    try {
      const project = await commands.getProject();
      set({ project });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unable to load project';
      set({ notice: { kind: 'error', message } });
    }
  },

  saveProject: async () => {
    try {
      await commands.saveProject();
      set({ notice: { kind: 'success', message: 'Project saved.' } });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unable to save project';
      set({ notice: { kind: 'error', message } });
    }
  },

  setActiveBank: async (index) => {
    try {
      await commands.setActiveBank(index);
      set((s) => {
        if (!s.project) return s;
        return { project: { ...s.project, active_bank: index } };
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unable to switch bank';
      set({ notice: { kind: 'error', message } });
    }
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
      const message = e instanceof Error ? e.message : 'Unable to trigger pad';
      set({ notice: { kind: 'error', message } });
    }
  },

  stopPad: async (bank, slot) => {
    try {
      await commands.stopPad(bank, slot);
      set((s) => {
        const next = new Set(s.activePads);
        next.delete(`${bank}-${slot}`);
        return { activePads: next };
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unable to stop pad';
      set({ notice: { kind: 'error', message } });
    }
  },

  stopAllPads: async () => {
    try {
      await commands.stopAllPads();
      set({ activePads: new Set() });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unable to stop all pads';
      set({ notice: { kind: 'error', message } });
    }
  },

  startRecording: async () => {
    try {
      await commands.startRecording();
      set({ recordingState: 'recording', notice: { kind: 'success', message: 'Recording started.' } });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unable to start recording';
      set({ notice: { kind: 'error', message } });
    }
  },

  stopRecording: async () => {
    try {
      await commands.stopRecording();
      set({ recordingState: 'idle', recordingDuration: 0, notice: { kind: 'success', message: 'Recording stopped.' } });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unable to stop recording';
      set({ notice: { kind: 'error', message } });
    }
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
  setNotice: (kind, message) => set({ notice: { kind, message } }),
  clearNotice: () => set({ notice: null }),
}));
