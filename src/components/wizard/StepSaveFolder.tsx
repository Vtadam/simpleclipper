import React, { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Button } from "../shared/Button";
import { openFolderDialog, getDefaultSaveFolder } from "../../lib/tauri";

interface Props {
  folder: string;
  onFolderChange: (folder: string) => void;
  onNext: () => void;
}

export function StepSaveFolder({ folder, onFolderChange, onNext }: Props) {
  const [picking, setPicking] = useState(false);

  // Auto-populate with the default save folder if not already set
  useEffect(() => {
    if (!folder) {
      getDefaultSaveFolder().then((defaultFolder) => {
        onFolderChange(defaultFolder);
      }).catch(() => {});
    }
  }, []);

  const pick = async () => {
    setPicking(true);
    try {
      const result = await openFolderDialog();
      if (result) onFolderChange(result);
    } finally {
      setPicking(false);
    }
  };

  return (
    <div className="flex flex-col flex-1 px-5 py-4 gap-4">
      <div>
        <p className="text-xs font-semibold uppercase tracking-widest text-[var(--accent)] mb-1">
          Step 1 of 4
        </p>
        <h2 className="text-base font-bold text-[var(--text)] mb-1">
          Where should we save your clips?
        </h2>
        <p className="text-[var(--text-muted)] text-xs">
          Pick a folder or use the default. All your clips will be saved here.
        </p>
      </div>

      <div className="flex-1 flex flex-col gap-3 justify-center">
        <motion.button
          whileTap={{ scale: 0.98 }}
          onClick={pick}
          disabled={picking}
          className="flex items-center gap-3 p-3 rounded-xl border border-[var(--border)] bg-[var(--surface)] hover:border-[var(--accent)] transition-colors text-left cursor-pointer"
        >
          <div className="w-8 h-8 rounded-lg bg-[var(--surface-2)] flex items-center justify-center text-base flex-shrink-0">
            📁
          </div>
          <div className="min-w-0 flex-1">
            <p className="text-xs text-[var(--text-muted)] mb-0.5">Save clips to</p>
            <p className="text-xs font-medium text-[var(--text)] truncate">
              {folder || "Loading default..."}
            </p>
          </div>
          <span className="text-[var(--text-muted)] text-xs flex-shrink-0">
            {picking ? "..." : "Change"}
          </span>
        </motion.button>
      </div>

      <Button size="md" className="w-full" onClick={onNext} disabled={!folder}>
        Got it →
      </Button>
    </div>
  );
}
