import React from "react";
import type { ClipMeta } from "../../types";
import { ClipCard } from "./ClipCard";

interface Props {
  clips: ClipMeta[];
  loading: boolean;
}

export function ClipList({ clips, loading }: Props) {
  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--text-muted)] text-sm">
        Loading clips...
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-2 text-center px-6">
        <span className="text-3xl">🎮</span>
        <p className="text-sm text-[var(--text-muted)]">
          No clips yet. Go play something awesome.
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto flex flex-col gap-1.5 px-3 pb-3">
      {clips.map((clip) => (
        <ClipCard key={clip.path} clip={clip} />
      ))}
    </div>
  );
}
