import React from "react";
import type { ClipMeta } from "../../types";
import { formatDuration, formatFileSize, formatTimestamp } from "../../lib/formatters";
import { openFileInExplorer } from "../../lib/tauri";

interface Props {
  clip: ClipMeta;
}

export function ClipCard({ clip }: Props) {
  return (
    <div className="flex items-center gap-2 p-2 rounded-lg bg-[var(--surface)] border border-[var(--border)] hover:border-[var(--accent)] transition-colors group">
      <div className="w-14 h-9 rounded-md bg-[var(--surface-2)] flex-shrink-0 overflow-hidden">
        {clip.thumbnail_b64 ? (
          <img
            src={`data:image/jpeg;base64,${clip.thumbnail_b64}`}
            alt="clip thumbnail"
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-lg">🎬</div>
        )}
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-xs font-medium text-[var(--text)] truncate leading-tight">
          {formatDuration(clip.duration_secs)} · {formatFileSize(clip.size_bytes)}
        </p>
        <p className="text-xs text-[var(--text-muted)] leading-tight">{formatTimestamp(clip.timestamp)}</p>
      </div>
      <button
        onClick={() => openFileInExplorer(clip.path)}
        className="opacity-0 group-hover:opacity-100 transition-opacity text-[var(--text-muted)] hover:text-[var(--accent)] text-sm"
        title="Open in Explorer"
      >
        📂
      </button>
    </div>
  );
}
