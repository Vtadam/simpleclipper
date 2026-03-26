import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Titlebar } from "../components/shared/Titlebar";
import { StatusBadge } from "../components/main/StatusBadge";
import { ClipList } from "../components/main/ClipList";
import { Button } from "../components/shared/Button";
import { useClips } from "../hooks/useClips";
import { getCaptureStatus, onClipSaved, onBufferUpdated, onHotkeyTriggered } from "../lib/tauri";
import { AnimatePresence, motion } from "framer-motion";

export function Main() {
  const navigate = useNavigate();
  const { clips, loading, refresh } = useClips();
  const [running, setRunning] = useState(false);
  const [bufferSecs, setBufferSecs] = useState(0);
  const [savedNotif, setSavedNotif] = useState<string | null>(null);

  useEffect(() => {
    refresh();
    getCaptureStatus().then((s) => setRunning(s.running)).catch(() => {});

    const unsubs = [
      onClipSaved((e) => {
        refresh();
        setSavedNotif(`Clip saved! ${Math.round(e.duration_secs)}s`);
        setTimeout(() => setSavedNotif(null), 3000);
      }),
      onBufferUpdated((e) => setBufferSecs(e.buffer_secs)),
      onHotkeyTriggered(() => {}),
    ];

    return () => {
      unsubs.forEach((p) => p.then((u) => u()));
    };
  }, []);

  return (
    <div className="flex flex-col h-screen" style={{ background: "var(--bg)" }}>
      <Titlebar />

      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border)]">
        <StatusBadge running={running} bufferSecs={bufferSecs} />
        <Button variant="ghost" size="sm" onClick={() => navigate("/settings")}>
          ⚙
        </Button>
      </div>

      <div className="px-3 pt-2 pb-1">
        <p className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
          Recent Clips
        </p>
      </div>

      <ClipList clips={clips} loading={loading} />

      <AnimatePresence>
        {savedNotif && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 10 }}
            className="absolute bottom-6 left-1/2 -translate-x-1/2 px-4 py-2 rounded-full bg-[var(--accent)] text-white text-sm font-medium shadow-lg"
          >
            {savedNotif}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
