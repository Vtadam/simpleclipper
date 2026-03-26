import React from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function Titlebar() {
  const hide = () => getCurrentWindow().hide();

  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between px-3 h-8 select-none flex-shrink-0"
      style={{ background: "var(--surface)" }}
    >
      <div
        data-tauri-drag-region
        className="flex items-center gap-2 text-xs font-semibold text-[var(--text)]"
      >
        <div className="w-2 h-2 rounded-full bg-[var(--accent)]" />
        SimpleClipper
      </div>
      <button
        onClick={hide}
        className="w-5 h-5 rounded-full flex items-center justify-center text-[var(--text-muted)] hover:bg-[var(--surface-2)] hover:text-[var(--text)] transition-colors text-xs"
        title="Minimize to tray"
      >
        ✕
      </button>
    </div>
  );
}
