export interface KeybindEntry {
  duration_secs: number;
  shortcut: string | null;
}

export type CaptureSource = "FullScreen" | "Window";
export type QualityPreset = "Low" | "Medium" | "High";
export type Theme = "Light" | "Dark" | "System";

export interface AppConfig {
  first_run_complete: boolean;
  save_folder: string;
  capture_source: CaptureSource;
  target_window_title: string | null;
  quality: QualityPreset;
  theme: Theme;
  start_with_windows: boolean;
  keybinds: KeybindEntry[];
}

export interface ClipMeta {
  path: string;
  thumbnail_b64: string | null;
  duration_secs: number;
  size_bytes: number;
  timestamp: string;
  name: string;
}

export interface CaptureStatus {
  running: boolean;
  source: CaptureSource;
  fps: number;
  buffer_secs: number;
}

export interface WindowInfo {
  hwnd: number;
  title: string;
  exe: string;
}

export interface ClipSavedEvent {
  path: string;
  duration_secs: number;
  size_bytes: number;
}

export const DEFAULT_KEYBINDS: KeybindEntry[] = [
  { duration_secs: 30, shortcut: null },
  { duration_secs: 60, shortcut: null },
  { duration_secs: 300, shortcut: null },
];

export const DURATION_LABELS: Record<number, string> = {
  10: "10 seconds",
  30: "30 seconds",
  60: "1 minute",
  120: "2 minutes",
  300: "5 minutes",
  600: "10 minutes",
  900: "15 minutes",
  1200: "20 minutes",
  1800: "30 minutes",
};

export const ALL_DURATIONS = [10, 30, 60, 120, 300, 600, 900, 1200, 1800];
