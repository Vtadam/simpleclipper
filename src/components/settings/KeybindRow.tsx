import React from "react";
import { formatDuration } from "../../lib/formatters";
import type { KeybindEntry } from "../../types";

interface Props {
  entry: KeybindEntry;
  capturing: boolean;
  onCapture: () => void;
  onClear: () => void;
}

export function KeybindRow({ entry, capturing, onCapture, onClear }: Props) {
  return (
    <div className="flex items-center gap-3 py-2">
      <span className="text-sm text-[var(--text)] w-28 flex-shrink-0">
        {formatDuration(entry.duration_secs)}
      </span>
      <button
        onClick={onCapture}
        className={`flex-1 text-sm px-3 py-1.5 rounded-lg border text-left transition-colors ${
          capturing
            ? "border-[var(--accent)] bg-[var(--surface-2)] text-[var(--accent)]"
            : "border-[var(--border)] bg-[var(--surface-2)] text-[var(--text-muted)] hover:border-[var(--accent)]"
        }`}
      >
        {capturing ? "Press keys now..." : entry.shortcut ?? "— not set —"}
      </button>
      {entry.shortcut && (
        <button
          onClick={onClear}
          className="text-xs text-[var(--text-muted)] hover:text-red-400 transition-colors"
        >
          ✕
        </button>
      )}
    </div>
  );
}
