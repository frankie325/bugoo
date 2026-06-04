import { create } from "zustand";

import { setSetting } from "../lib/api/settings";

interface SettingsState {
  settings: Record<string, string>;
  setSettings: (settings: Record<string, string>) => void;
  updateSetting: (key: string, value: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: {},
  setSettings: (settings) => set({ settings }),
  updateSetting: (key, value) => {
    set((state) => ({
      settings: { ...state.settings, [key]: value },
    }));
    return setSetting(key, value).catch((error) => {
      console.warn(`Failed to persist setting ${key}:`, error);
    });
  },
}));
