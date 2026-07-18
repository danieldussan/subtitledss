import { useState } from "react";
import { AppConfig } from "../../hooks/useSettings";
import { AudioSettings } from "./AudioSettings";
import { WhisperSettings } from "./WhisperSettings";
import { TranslationSettings } from "./TranslationSettings";
import { ThemeSettings } from "./ThemeSettings";
import { ShortcutsSettings } from "./ShortcutsSettings";
import { AiSettings } from "./AiSettings";

type SettingsFilter = "general" | "appearance" | "advanced";

interface SettingsLayoutProps {
  config: AppConfig;
  onSave: (config: AppConfig) => Promise<void>;
  isCapturing: boolean;
}

export function SettingsLayout({ config, onSave, isCapturing }: SettingsLayoutProps) {
  const [filter, setFilter] = useState<SettingsFilter>("general");

  const filters: { id: SettingsFilter; label: string }[] = [
    { id: "general", label: "General" },
    { id: "appearance", label: "Appearance" },
    { id: "advanced", label: "Advanced" },
  ];

  return (
    <div className="h-full flex flex-col">
      <div className="px-5 pt-4 pb-0">
        <div className="tab-bar w-fit">
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id)}
              className={`tab-item ${filter === f.id ? "active" : ""}`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-5 py-4">
        <div className="max-w-3xl mx-auto">
          {filter === "general" && (
            <div>
              <AudioSettings config={config} onSave={onSave} isCapturing={isCapturing} />
              <WhisperSettings config={config} onSave={onSave} />
              <TranslationSettings config={config} onSave={onSave} />
              <AiSettings config={config} onSave={onSave} />
            </div>
          )}
          {filter === "appearance" && (
            <div>
              <ThemeSettings config={config} onSave={onSave} />
            </div>
          )}
          {filter === "advanced" && (
            <div>
              <ShortcutsSettings config={config} onSave={onSave} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
