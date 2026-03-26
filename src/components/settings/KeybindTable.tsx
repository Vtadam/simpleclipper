import React, { useState } from "react";
import { KeybindRow } from "./KeybindRow";
import { captureNextKeypress } from "../../lib/tauri";
import type { KeybindEntry } from "../../types";
import { ALL_DURATIONS } from "../../types";
import { Button } from "../shared/Button";
import { formatDuration } from "../../lib/formatters";

interface Props {
  keybinds: KeybindEntry[];
  onChange: (keybinds: KeybindEntry[]) => void;
}

export function KeybindTable({ keybinds, onChange }: Props) {
  const [capturingIndex, setCapturingIndex] = useState<number | null>(null);
  const [showAdd, setShowAdd] = useState(false);

  const usedDurations = new Set(keybinds.map((k) => k.duration_secs));
  const availableDurations = ALL_DURATIONS.filter((d) => !usedDurations.has(d));

  const captureFor = async (idx: number) => {
    setCapturingIndex(idx);
    try {
      const shortcut = await captureNextKeypress();
      onChange(keybinds.map((k, i) => (i === idx ? { ...k, shortcut } : k)));
    } finally {
      setCapturingIndex(null);
    }
  };

  const clear = (idx: number) => {
    onChange(keybinds.map((k, i) => (i === idx ? { ...k, shortcut: null } : k)));
  };

  const addDuration = (secs: number) => {
    onChange([...keybinds, { duration_secs: secs, shortcut: null }]);
    setShowAdd(false);
  };

  const remove = (idx: number) => {
    onChange(keybinds.filter((_, i) => i !== idx));
  };

  return (
    <div className="flex flex-col gap-1">
      {keybinds.map((entry, idx) => (
        <div key={entry.duration_secs} className="flex items-center gap-2">
          <div className="flex-1">
            <KeybindRow
              entry={entry}
              capturing={capturingIndex === idx}
              onCapture={() => captureFor(idx)}
              onClear={() => clear(idx)}
            />
          </div>
          <button
            onClick={() => remove(idx)}
            className="text-xs text-[var(--text-muted)] hover:text-red-400 transition-colors pb-0.5"
            title="Remove"
          >
            −
          </button>
        </div>
      ))}

      {availableDurations.length > 0 && (
        <div className="mt-2">
          {showAdd ? (
            <div className="flex flex-wrap gap-2">
              {availableDurations.map((d) => (
                <button
                  key={d}
                  onClick={() => addDuration(d)}
                  className="text-xs px-3 py-1.5 rounded-lg border border-[var(--border)] bg-[var(--surface)] hover:border-[var(--accent)] text-[var(--text-muted)] transition-colors"
                >
                  + {formatDuration(d)}
                </button>
              ))}
            </div>
          ) : (
            <Button variant="ghost" size="sm" onClick={() => setShowAdd(true)}>
              + Add duration
            </Button>
          )}
        </div>
      )}
    </div>
  );
}
