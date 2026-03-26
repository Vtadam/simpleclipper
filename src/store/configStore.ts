import { create } from "zustand";
import type { AppConfig } from "../types";
import { DEFAULT_KEYBINDS } from "../types";
import { getConfig, saveConfig } from "../lib/tauri";

interface ConfigStore {
  config: AppConfig | null;
  loading: boolean;
  load: () => Promise<void>;
  update: (partial: Partial<AppConfig>) => Promise<void>;
}

const DEFAULT_CONFIG: AppConfig = {
  first_run_complete: false,
  save_folder: "",
  capture_source: "FullScreen",
  target_window_title: null,
  quality: "Medium",
  theme: "Dark",
  start_with_windows: false,
  keybinds: DEFAULT_KEYBINDS,
};

export const useConfigStore = create<ConfigStore>((set, get) => ({
  config: null,
  loading: true,
  load: async () => {
    try {
      const config = await getConfig();
      set({ config, loading: false });
    } catch {
      set({ config: DEFAULT_CONFIG, loading: false });
    }
  },
  update: async (partial) => {
    const current = get().config;
    if (!current) return;
    const next = { ...current, ...partial };
    set({ config: next });
    await saveConfig(next);
  },
}));
