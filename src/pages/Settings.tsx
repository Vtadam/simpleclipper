import React from "react";
import { useNavigate } from "react-router-dom";
import { Titlebar } from "../components/shared/Titlebar";
import { Button } from "../components/shared/Button";
import { Toggle } from "../components/shared/Toggle";
import { FolderPicker } from "../components/settings/FolderPicker";
import { SourceSelector } from "../components/settings/SourceSelector";
import { QualityPicker } from "../components/settings/QualityPicker";
import { KeybindTable } from "../components/settings/KeybindTable";
import { useConfig } from "../hooks/useConfig";
import { registerKeybinds, setStartWithWindows } from "../lib/tauri";

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-2">
      <p className="text-xs font-semibold uppercase tracking-widest text-[var(--text-muted)]">
        {title}
      </p>
      {children}
    </div>
  );
}

export function Settings() {
  const navigate = useNavigate();
  const { config, update } = useConfig();

  if (!config) return null;

  return (
    <div className="flex flex-col h-screen" style={{ background: "var(--bg)" }}>
      <Titlebar />

      <div className="flex items-center gap-3 px-4 py-3 border-b border-[var(--border)]">
        <button
          onClick={() => navigate("/main")}
          className="text-[var(--text-muted)] hover:text-[var(--text)] transition-colors text-sm"
        >
          ← Back
        </button>
        <p className="text-sm font-semibold text-[var(--text)]">Settings</p>
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-4 flex flex-col gap-6">
        <Section title="Keybinds">
          <KeybindTable
            keybinds={config.keybinds}
            onChange={async (keybinds) => {
              await update({ keybinds });
              await registerKeybinds(keybinds);
            }}
          />
        </Section>

        <Section title="Save Location">
          <FolderPicker
            folder={config.save_folder}
            onChange={(save_folder) => update({ save_folder })}
          />
        </Section>

        <Section title="Capture">
          <SourceSelector
            value={config.capture_source}
            onChange={(capture_source) => update({ capture_source })}
          />
        </Section>

        <Section title="Quality">
          <QualityPicker
            value={config.quality}
            onChange={(quality) => update({ quality })}
          />
          <p className="text-xs text-[var(--text-muted)]">
            {config.quality === "Low" && "~4 Mbps · Lower file sizes, good for most games"}
            {config.quality === "Medium" && "~8 Mbps · Recommended balance of quality and size"}
            {config.quality === "High" && "~16 Mbps · Best quality · uses more RAM for long clips"}
          </p>
        </Section>

        <Section title="System">
          <Toggle
            checked={config.start_with_windows}
            label="Start with Windows"
            onChange={async (v) => {
              await update({ start_with_windows: v });
              await setStartWithWindows(v);
            }}
          />
        </Section>
      </div>
    </div>
  );
}
