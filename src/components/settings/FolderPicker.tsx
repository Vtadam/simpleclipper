import React, { useState } from "react";
import { openFolderDialog } from "../../lib/tauri";

interface Props {
  folder: string;
  onChange: (folder: string) => void;
}

export function FolderPicker({ folder, onChange }: Props) {
  const [picking, setPicking] = useState(false);

  const pick = async () => {
    setPicking(true);
    const result = await openFolderDialog();
    if (result) onChange(result);
    setPicking(false);
  };

  return (
    <button
      onClick={pick}
      disabled={picking}
      className="flex items-center gap-3 w-full p-3 rounded-xl border border-[var(--border)] bg-[var(--surface)] hover:border-[var(--accent)] transition-colors text-left"
    >
      <span className="text-lg">📁</span>
      <span className="text-sm text-[var(--text)] truncate flex-1 min-w-0">
        {folder || "Choose folder..."}
      </span>
      <span className="text-xs text-[var(--text-muted)] flex-shrink-0">
        {picking ? "..." : "Change →"}
      </span>
    </button>
  );
}
