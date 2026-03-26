import type {
  AppConfig,
  ClipMeta,
  CaptureStatus,
  WindowInfo,
  KeybindEntry,
  ClipSavedEvent,
} from "../types";

// Detect whether we're running inside Tauri or just a browser
const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

// ── Mock data for browser preview ────────────────────────────────────────────

let mockConfig: AppConfig = {
  first_run_complete: false,
  save_folder: "C:\\Users\\You\\Videos\\SimpleClipper",
  capture_source: "FullScreen",
  target_window_title: null,
  quality: "Medium",
  theme: "Dark",
  start_with_windows: false,
  keybinds: [
    { duration_secs: 30, shortcut: "Ctrl+Shift+F1" },
    { duration_secs: 60, shortcut: "Ctrl+Shift+F2" },
    { duration_secs: 300, shortcut: null },
  ],
};

const mockClips: ClipMeta[] = [
  {
    path: "C:\\Users\\You\\Videos\\SimpleClipper\\clip_20250325_143012_30s.mp4",
    thumbnail_b64: null,
    duration_secs: 30,
    size_bytes: 28_000_000,
    timestamp: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
    name: "clip_20250325_143012_30s",
  },
  {
    path: "C:\\Users\\You\\Videos\\SimpleClipper\\clip_20250325_141500_60s.mp4",
    thumbnail_b64: null,
    duration_secs: 60,
    size_bytes: 58_000_000,
    timestamp: new Date(Date.now() - 1000 * 60 * 20).toISOString(),
    name: "clip_20250325_141500_60s",
  },
  {
    path: "C:\\Users\\You\\Videos\\SimpleClipper\\clip_20250325_133022_300s.mp4",
    thumbnail_b64: null,
    duration_secs: 300,
    size_bytes: 290_000_000,
    timestamp: new Date(Date.now() - 1000 * 60 * 60).toISOString(),
    name: "clip_20250325_133022_300s",
  },
];

// ── Tauri invoke/listen wrappers ──────────────────────────────────────────────

async function inv<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(command, args);
  }
  throw new Error(`[mock] No handler for command: ${command}`);
}

type Unlistener = () => void;

async function lst<T>(
  event: string,
  _cb: (payload: T) => void
): Promise<Unlistener> {
  if (isTauri) {
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<T>(event, (e) => _cb(e.payload));
    return unlisten;
  }
  // In browser: return a no-op unlistener
  return () => {};
}

// ── Config ────────────────────────────────────────────────────────────────────

export const getConfig = async (): Promise<AppConfig> => {
  if (!isTauri) return { ...mockConfig };
  return inv<AppConfig>("get_config");
};

export const saveConfig = async (config: AppConfig): Promise<void> => {
  if (!isTauri) { mockConfig = { ...config }; return; }
  return inv<void>("save_config", { config });
};

// ── File System ───────────────────────────────────────────────────────────────

export const openFolderDialog = async (): Promise<string | null> => {
  if (!isTauri) {
    // Simulate a folder pick
    return "C:\\Users\\You\\Videos\\SimpleClipper";
  }
  return inv<string | null>("open_folder_dialog");
};

export const openFileInExplorer = async (path: string): Promise<void> => {
  if (!isTauri) { console.log("[mock] open in explorer:", path); return; }
  return inv<void>("open_file_in_explorer", { path });
};

export const getClipsList = async (): Promise<ClipMeta[]> => {
  if (!isTauri) return [...mockClips];
  return inv<ClipMeta[]>("get_clips_list");
};

// ── Capture ───────────────────────────────────────────────────────────────────

export const startCapture = async (): Promise<void> => {
  if (!isTauri) { console.log("[mock] capture started"); return; }
  return inv<void>("start_capture");
};

export const stopCapture = async (): Promise<void> => {
  if (!isTauri) { console.log("[mock] capture stopped"); return; }
  return inv<void>("stop_capture");
};

export const getCaptureStatus = async (): Promise<CaptureStatus> => {
  if (!isTauri) return { running: true, source: "FullScreen", fps: 30, buffer_secs: 47 };
  return inv<CaptureStatus>("get_capture_status");
};

// ── Hotkeys ───────────────────────────────────────────────────────────────────

export const registerKeybinds = async (entries: KeybindEntry[]): Promise<void> => {
  if (!isTauri) { console.log("[mock] keybinds registered:", entries); return; }
  return inv<void>("register_keybinds", { entries });
};

/**
 * Capture the next keypress using DOM events only.
 * This always uses the DOM approach regardless of Tauri/browser context
 * because the Rust backend no longer implements this functionality.
 */
export const captureNextKeypress = async (): Promise<string> => {
  return new Promise((resolve) => {
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const parts: string[] = [];
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");
      const key = e.key;
      if (!["Control", "Shift", "Alt", "Meta"].includes(key)) {
        parts.push(key.length === 1 ? key.toUpperCase() : key);
      }
      if (parts.length >= 2) {
        window.removeEventListener("keydown", handler, true);
        resolve(parts.join("+"));
      }
    };
    window.addEventListener("keydown", handler, true);
  });
};

// ── System ────────────────────────────────────────────────────────────────────

export const setStartWithWindows = async (enabled: boolean): Promise<void> => {
  if (!isTauri) { console.log("[mock] start with windows:", enabled); return; }
  return inv<void>("set_start_with_windows", { enabled });
};

export const getWindowsList = async (): Promise<WindowInfo[]> => {
  if (!isTauri) return [
    { hwnd: 1, title: "Fortnite", exe: "FortniteClient-Win64-Shipping.exe" },
    { hwnd: 2, title: "Chrome", exe: "chrome.exe" },
  ];
  return inv<WindowInfo[]>("get_windows_list");
};

// ── Events ────────────────────────────────────────────────────────────────────

export const onClipSaved = (cb: (payload: ClipSavedEvent) => void) =>
  lst<ClipSavedEvent>("clip-saved", cb);

export const onCaptureError = (cb: (message: string) => void) =>
  lst<{ message: string }>("capture-error", (e) => cb(e.message));

export const onBufferUpdated = (
  cb: (payload: { buffer_secs: number; buffer_bytes: number }) => void
) => lst<{ buffer_secs: number; buffer_bytes: number }>("buffer-updated", cb);

export const onHotkeyTriggered = (cb: (duration_secs: number) => void) =>
  lst<{ duration_secs: number }>("hotkey-triggered", (e) => cb(e.duration_secs));
