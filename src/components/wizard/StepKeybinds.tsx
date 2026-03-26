import React, { useState, useCallback } from "react";
import { Button } from "../shared/Button";
import { captureNextKeypress } from "../../lib/tauri";
import { formatDuration } from "../../lib/formatters";
import type { KeybindEntry } from "../../types";

interface Props {
  keybinds: KeybindEntry[];
  onKeybindsChange: (keybinds: KeybindEntry[]) => void;
  onNext: () => void;
}

export function StepKeybinds({ keybinds, onKeybindsChange, onNext }: Props) {
  const [capturingIndex, setCapturingIndex] = useState<number | null>(null);

  const captureFor = useCallback(async (idx: number) => {
    setCapturingIndex(idx);
    try {
      const shortcut = await captureNextKeypress();
      if (shortcut) {
        const next = keybinds.map((k, i) =>
          i === idx ? { ...k, shortcut } : k
        );
        onKeybindsChange(next);
      }
    } finally {
      setCapturingIndex(null);
    }
  }, [keybinds, onKeybindsChange]);

  const clear = (idx: number) => {
    onKeybindsChange(keybinds.map((k, i) => (i === idx ? { ...k, shortcut: null } : k)));
  };

  return (
    <div className="flex flex-col flex-1 px-5 py-4 gap-3">
      <div>
        <p className="text-xs font-semibold uppercase tracking-widest text-[var(--accent)] mb-1">
          Step 3 of 4
        </p>
        <h2 className="text-base font-bold text-[var(--text)] mb-0.5">
          Set your clip keys
        </h2>
        <p className="text-[var(--text-muted)] text-xs">
          Press a key combo (e.g. Ctrl+F1) or a function key (e.g. F9). Press Esc to cancel.
        </p>
      </div>

      <div className="flex flex-col gap-1.5 flex-1 overflow-y-auto min-h-0">
        {keybinds.map((entry, idx) => (
          <div
            key={idx}
            className="flex items-center gap-2 px-2.5 py-2 rounded-lg bg-[var(--surface)] border border-[var(--border)]"
          >
            <span className="text-xs text-[var(--text)] w-16 flex-shrink-0 font-medium">
              {formatDuration(entry.duration_secs)}
            </span>
            <button
              onClick={() => captureFor(idx)}
              className={`flex-1 text-xs px-2 py-1.5 rounded-md border text-left transition-colors truncate ${
                capturingIndex === idx
                  ? "border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)] animate-pulse"
                  : "border-[var(--border)] bg-[var(--surface-2)] text-[var(--text-muted)] hover:border-[var(--accent)]"
              }`}
            >
              {capturingIndex === idx
                ? "Press keys now..."
                : entry.shortcut ?? "Click to set"}
            </button>
            {entry.shortcut && (
              <button
                onClick={() => clear(idx)}
                className="text-[var(--text-muted)] hover:text-red-400 transition-colors text-xs px-1"
              >
                ✕
              </button>
            )}
          </div>
        ))}
      </div>

      <p className="text-[var(--text-muted)] text-[10px] text-center">
        You can skip this and set keybinds later in settings.
      </p>

      <Button size="md" className="w-full flex-shrink-0" onClick={onNext}>
        Got it →
      </Button>
    </div>
  );
}
