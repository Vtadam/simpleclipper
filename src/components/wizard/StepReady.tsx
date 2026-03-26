import React from "react";
import { motion } from "framer-motion";
import { Button } from "../shared/Button";

interface Props {
  onFinish: () => void;
}

export function StepReady({ onFinish }: Props) {
  return (
    <div className="flex flex-col flex-1 items-center justify-center px-6 text-center gap-5">
      <motion.div
        initial={{ scale: 0, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 200, damping: 18, delay: 0.1 }}
        className="w-16 h-16 rounded-full bg-[var(--accent)] flex items-center justify-center text-3xl"
      >
        ✓
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.35 }}
        className="flex flex-col gap-2"
      >
        <h2 className="text-xl font-bold text-[var(--text)]">You're ready!</h2>
        <p className="text-[var(--text-muted)] text-xs leading-relaxed">
          SimpleClipper is now in your system tray.
          <br />
          Play your game — press your key to save the clip.
        </p>
      </motion.div>

      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.55 }}
        className="w-full"
      >
        <Button size="md" className="w-full" onClick={onFinish}>
          Start Clipping 🎬
        </Button>
      </motion.div>
    </div>
  );
}
