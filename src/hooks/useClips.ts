import { useState, useCallback } from "react";
import type { ClipMeta } from "../types";
import { getClipsList } from "../lib/tauri";

export function useClips() {
  const [clips, setClips] = useState<ClipMeta[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const list = await getClipsList();
      setClips(list.sort((a, b) => b.timestamp.localeCompare(a.timestamp)));
    } finally {
      setLoading(false);
    }
  }, []);

  return { clips, loading, refresh };
}
