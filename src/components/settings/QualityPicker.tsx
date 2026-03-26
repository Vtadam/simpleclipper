import React from "react";
import type { QualityPreset } from "../../types";

interface Props {
  value: QualityPreset;
  onChange: (value: QualityPreset) => void;
}

const QUALITY_INFO: Record<QualityPreset, { label: string; desc: string }> = {
  Low: { label: "Low", desc: "4 Mbps" },
  Medium: { label: "Medium", desc: "8 Mbps" },
  High: { label: "High", desc: "16 Mbps" },
};

export function QualityPicker({ value, onChange }: Props) {
  return (
    <div className="flex gap-2">
      {(["Low", "Medium", "High"] as QualityPreset[]).map((q) => (
        <button
          key={q}
          onClick={() => onChange(q)}
          className={`flex-1 py-2 px-3 rounded-xl text-sm border transition-colors text-center ${
            value === q
              ? "border-[var(--accent)] bg-[var(--surface)] text-[var(--text)] font-medium"
              : "border-[var(--border)] bg-[var(--surface)] text-[var(--text-muted)] hover:border-[var(--accent)]"
          }`}
        >
          <div>{QUALITY_INFO[q].label}</div>
          <div className="text-xs opacity-60">{QUALITY_INFO[q].desc}</div>
        </button>
      ))}
    </div>
  );
}
