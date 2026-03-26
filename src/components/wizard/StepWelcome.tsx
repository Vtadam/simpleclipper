import React from "react";
import { motion } from "framer-motion";
import { Button } from "../shared/Button";

interface Props {
  onNext: () => void;
}

export function StepWelcome({ onNext }: Props) {
  return (
    <div className="flex flex-col items-center justify-center flex-1 px-6 text-center gap-5">
      <motion.div
        initial={{ scale: 0.6, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ delay: 0.1, duration: 0.4, ease: "easeOut" }}
        className="w-14 h-14 rounded-2xl bg-[var(--accent)] flex items-center justify-center text-2xl shadow-2xl"
      >
        ✂️
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.25, duration: 0.35 }}
        className="flex flex-col gap-1.5"
      >
        <h1 className="text-xl font-bold text-[var(--text)] tracking-tight">
          SimpleClipper
        </h1>
        <p className="text-[var(--text-muted)] text-xs leading-relaxed">
          Clip your gameplay instantly.<br />No setup. No fuss. Just press a key.
        </p>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.45, duration: 0.3 }}
        className="flex flex-col gap-3 w-full"
      >
        <div className="flex flex-col gap-1.5 text-xs text-[var(--text-muted)] text-left">
          {["Always recording in the background", "Press a key → clip saved instantly", "Ultra lightweight — barely uses any RAM"].map(
            (text, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.5 + i * 0.08 }}
                className="flex items-center gap-2"
              >
                <div className="w-1.5 h-1.5 rounded-full bg-[var(--accent)] flex-shrink-0" />
                {text}
              </motion.div>
            )
          )}
        </div>
        <Button size="md" className="mt-2 w-full" onClick={onNext}>
          Let's set it up →
        </Button>
      </motion.div>
    </div>
  );
}
