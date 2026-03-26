import React from "react";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
}

export function Toggle({ checked, onChange, label }: ToggleProps) {
  return (
    <label className="flex items-center gap-3 cursor-pointer">
      <div
        onClick={() => onChange(!checked)}
        className={`relative w-10 h-6 rounded-full transition-colors duration-200 ${
          checked ? "bg-[var(--accent)]" : "bg-[var(--surface-2)]"
        }`}
      >
        <div
          className={`absolute top-1 left-1 w-4 h-4 bg-white rounded-full shadow transition-transform duration-200 ${
            checked ? "translate-x-4" : "translate-x-0"
          }`}
        />
      </div>
      {label && (
        <span className="text-sm text-[var(--text)]">{label}</span>
      )}
    </label>
  );
}
