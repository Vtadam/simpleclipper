import React from "react";

interface Props {
  running: boolean;
  bufferSecs: number;
}

export function StatusBadge({ running, bufferSecs }: Props) {
  return (
    <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-[var(--surface-2)] border border-[var(--border)]">
      <div className="relative flex items-center justify-center">
        <div
          className={`w-2 h-2 rounded-full ${
            running ? "bg-green-400" : "bg-[var(--text-muted)]"
          }`}
        />
        {running && (
          <div className="absolute w-2 h-2 rounded-full bg-green-400 animate-ping opacity-60" />
        )}
      </div>
      <span className="text-xs text-[var(--text-muted)] font-medium">
        {running
          ? bufferSecs > 0
            ? `${Math.round(bufferSecs)}s buffered`
            : "Recording..."
          : "Stopped"}
      </span>
    </div>
  );
}
