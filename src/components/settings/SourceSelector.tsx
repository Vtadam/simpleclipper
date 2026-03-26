import React from "react";
import type { CaptureSource } from "../../types";

interface Props {
  value: CaptureSource;
  onChange: (value: CaptureSource) => void;
}

export function SourceSelector({ value, onChange }: Props) {
  return (
    <div className="flex gap-2">
      {(["FullScreen", "Window"] as CaptureSource[]).map((opt) => (
        <button
          key={opt}
          onClick={() => onChange(opt)}
          className={`flex-1 py-2 px-3 rounded-xl text-sm border transition-colors ${
            value === opt
              ? "border-[var(--accent)] bg-[var(--surface)] text-[var(--text)] font-medium"
              : "border-[var(--border)] bg-[var(--surface)] text-[var(--text-muted)] hover:border-[var(--accent)]"
          }`}
        >
          {opt === "FullScreen" ? "🖥️ Full Screen" : "🪟 Specific App"}
        </button>
      ))}
    </div>
  );
}
