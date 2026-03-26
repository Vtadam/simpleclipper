import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import { WizardShell } from "../components/wizard/WizardShell";
import { StepWelcome } from "../components/wizard/StepWelcome";
import { StepSaveFolder } from "../components/wizard/StepSaveFolder";
import { StepCaptureSource } from "../components/wizard/StepCaptureSource";
import { StepKeybinds } from "../components/wizard/StepKeybinds";
import { StepReady } from "../components/wizard/StepReady";
import { useConfigStore } from "../store/configStore";
import type { CaptureSource, KeybindEntry } from "../types";
import { DEFAULT_KEYBINDS } from "../types";
import { startCapture, registerKeybinds } from "../lib/tauri";

const TOTAL_STEPS = 5;

export function Wizard() {
  const navigate = useNavigate();
  const { update } = useConfigStore();

  const [step, setStep] = useState(0);
  const [folder, setFolder] = useState("");
  const [source, setSource] = useState<CaptureSource>("FullScreen");
  const [keybinds, setKeybinds] = useState<KeybindEntry[]>(DEFAULT_KEYBINDS);

  const next = () => setStep((s) => Math.min(s + 1, TOTAL_STEPS - 1));

  const finish = async () => {
    await update({
      first_run_complete: true,
      save_folder: folder,
      capture_source: source,
      keybinds,
    });
    await registerKeybinds(keybinds);
    await startCapture();
    navigate("/main");
  };

  return (
    <WizardShell step={step} totalSteps={TOTAL_STEPS}>
      {step === 0 && <StepWelcome onNext={next} />}
      {step === 1 && (
        <StepSaveFolder folder={folder} onFolderChange={setFolder} onNext={next} />
      )}
      {step === 2 && (
        <StepCaptureSource source={source} onSourceChange={setSource} onNext={next} />
      )}
      {step === 3 && (
        <StepKeybinds keybinds={keybinds} onKeybindsChange={setKeybinds} onNext={next} />
      )}
      {step === 4 && <StepReady onFinish={finish} />}
    </WizardShell>
  );
}
