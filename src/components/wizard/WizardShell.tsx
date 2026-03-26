import React from "react";
import { motion, AnimatePresence } from "framer-motion";

interface WizardShellProps {
  step: number;
  totalSteps: number;
  children: React.ReactNode;
}

export function WizardShell({ step, totalSteps, children }: WizardShellProps) {
  return (
    <div className="flex flex-col h-screen" style={{ background: "var(--bg)" }}>
      <div className="flex-1 overflow-hidden relative">
        <AnimatePresence mode="wait">
          <motion.div
            key={step}
            initial={{ opacity: 0, x: 40 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -40 }}
            transition={{ duration: 0.22, ease: "easeOut" }}
            className="absolute inset-0 flex flex-col"
          >
            {children}
          </motion.div>
        </AnimatePresence>
      </div>
      <div className="flex justify-center gap-2 pb-4">
        {Array.from({ length: totalSteps }).map((_, i) => (
          <div
            key={i}
            className={`h-1.5 rounded-full transition-all duration-300 ${
              i === step
                ? "w-6 bg-[var(--accent)]"
                : i < step
                ? "w-1.5 bg-[var(--accent)] opacity-40"
                : "w-1.5 bg-[var(--border)]"
            }`}
          />
        ))}
      </div>
    </div>
  );
}
