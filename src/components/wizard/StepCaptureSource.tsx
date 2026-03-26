import React from "react";
import { Button } from "../shared/Button";
import type { CaptureSource } from "../../types";

interface Props {
  source: CaptureSource;
  onSourceChange: (source: CaptureSource) => void;
  onNext: () => void;
}

export function StepCaptureSource({ source, onSourceChange, onNext }: Props) {
  const options: { value: CaptureSource; icon: string; title: string; desc: string }[] = [
    {
      value: "FullScreen",
      icon: "🖥️",
      title: "Full Screen",
      desc: "Captures everything on your monitor. Best for most games.",
    },
    {
      value: "Window",
      icon: "🪟",
      title: "Specific App",
      desc: "Only captures one game window. Hides your desktop.",
    },
  ];

  return (
    <div className="flex flex-col flex-1 px-5 py-6 gap-5">
      <div>
        <p className="text-xs font-semibold uppercase tracking-widest text-[var(--accent)] mb-1">
          Step 2 of 4
        </p>
        <h2 className="text-base font-bold text-[var(--text)] mb-1">
          What do you want to capture?
        </h2>
        <p className="text-[var(--text-muted)] text-xs">
          You can always change this in settings later.
        </p>
      </div>

      <div className="flex flex-col gap-2 flex-1 justify-center">
        {options.map((opt) => (
          <button
            key={opt.value}
            onClick={() => onSourceChange(opt.value)}
            className={`flex items-center gap-3 p-3 rounded-xl border transition-all text-left ${
              source === opt.value
                ? "border-[var(--accent)] bg-[var(--surface)]"
                : "border-[var(--border)] bg-[var(--surface)] hover:border-[var(--accent)] hover:opacity-80"
            }`}
          >
            <div className="w-8 h-8 rounded-lg bg-[var(--surface-2)] flex items-center justify-center text-base flex-shrink-0">
              {opt.icon}
            </div>
            <div>
              <p className="text-xs font-semibold text-[var(--text)]">{opt.title}</p>
              <p className="text-xs text-[var(--text-muted)] mt-0.5">{opt.desc}</p>
            </div>
            {source === opt.value && (
              <div className="ml-auto w-4 h-4 rounded-full bg-[var(--accent)] flex items-center justify-center text-white text-xs">
                ✓
              </div>
            )}
          </button>
        ))}
      </div>

      <Button size="md" className="w-full" onClick={onNext}>
        Got it →
      </Button>
    </div>
  );
}
