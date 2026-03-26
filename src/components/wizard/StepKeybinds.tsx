import React, { useState } from "react";
import { Button } from "../shared/Button";
import { captureNextKeypress } from "../../lib/tauri";
import { formatDuration } from "../../lib/formatters";
import type { KeybindEntry } from "../../types";
import { DEFAULT_KEYBINDS } from "../../types";

interface Props {
  keybinds: KeybindEntry[];
  onKeybindsChange: (keybinds: KeybindEntry[]) => void;
  onNext: () => void;
}

export function StepKeybinds({ keybinds, onKeybindsChange, onNext }: Props) {
  const [capturingIndex, setCapturingIndex] = useState<number | null>(null);

  const captureFor = async (idx: number) => {
    setCapturingIndex(idx);
    try {
      const shortcut = await captureNextKeypress();
      const next = keybinds.map((k, i) =>
        i === idx ? { ...k, shortcut } : k
      );
      onKeybindsChange(next);
    } finally {
      setCapturingIndex(null);
    }
  };

  const clear = (idx: number) => {
    onKeybindsChange(keybinds.map((k, i) => (i === idx ? { ...k, shortcut: null } : k)));
  };

  return (
    <div className="flex flex-col flex-1 px-10 py-12 gap-6">
      <div>
        <p className="text-xs font-semibold uppercase tracking-widest text-[var(--accent)] mb-2">
          Step 3 of 4
        </p>
        <h2 className="text-2xl font-bold text-[var(--text)] mb-2">
          Set your clip keys
        </h2>
        <p className="text-[var(--text-muted)] text-sm">
          Each key saves the last N seconds. Press any key combo to assign.
        </p>
      </div>

      <div className="flex flex-col gap-2 flex-1 overflow-y-auto">
        {keybinds.map((entry, idx) => (
          <div
            key={idx}
            className="flex items-center gap-3 p-3 rounded-xl bg-[var(--surface)] border border-[var(--border)]"
          >
            <span className="text-sm text-[var(--text)] w-24 flex-shrink-0">
              {formatDuration(entry.duration_secs)}
            </span>
            <button
              onClick={() => captureFor(idx)}
              className={`flex-1 text-sm px-3 py-1.5 rounded-lg border text-left transition-colors ${
                capturingIndex === idx
                  ? "border-[var(--accent)] bg-[var(--surface-2)] text-[var(--accent)]"
                  : "border-[var(--border)] bg-[var(--surface-2)] text-[var(--text-muted)] hover:border-[var(--accent)]"
              }`}
            >
              {capturingIndex === idx
                ? "Press keys now..."
                : entry.shortcut ?? "Click to set key"}
            </button>
            {entry.shortcut && (
              <button
                onClick={() => clear(idx)}
                className="text-[var(--text-muted)] hover:text-red-400 transition-colors text-xs"
              >
                ✕
              </button>
            )}
          </div>
        ))}
      </div>

      <Button size="lg" className="w-full" onClick={onNext}>
        Got it →
      </Button>
    </div>
  );
}
