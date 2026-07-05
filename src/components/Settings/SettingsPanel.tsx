import { useState } from "react";
import { useSettings } from "../../hooks/useSettings";
import { AudioSettings } from "./AudioSettings";
import { WhisperSettings } from "./WhisperSettings";
import { ThemeSettings } from "./ThemeSettings";
import { ShortcutsSettings } from "./ShortcutsSettings";
import { Mic, Brain, Palette, Keyboard, Loader2 } from "lucide-react";

type SettingsTab = "audio" | "whisper" | "theme" | "shortcuts";

interface SettingsPanelProps {
  isCapturing: boolean;
}

export function SettingsPanel({ isCapturing }: SettingsPanelProps) {
  const { config, loading, error, saveConfig } = useSettings();
  const [activeTab, setActiveTab] = useState<SettingsTab>("audio");

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64 gap-2 text-text-muted">
        <Loader2 size={18} className="animate-spin" />
        <span>Loading settings...</span>
      </div>
    );
  }

  if (error || !config) {
    return (
      <div className="flex items-center justify-center h-64 text-danger">
        <span>Error loading settings: {error}</span>
      </div>
    );
  }

  const tabs: { id: SettingsTab; label: string; icon: React.ElementType }[] = [
    { id: "audio", label: "Audio", icon: Mic },
    { id: "whisper", label: "Whisper", icon: Brain },
    { id: "theme", label: "Overlay", icon: Palette },
    { id: "shortcuts", label: "Shortcuts", icon: Keyboard },
  ];

  return (
    <div className="h-full flex flex-col">
      <div className="px-5 pt-4 pb-0">
        <div className="tab-bar">
          {tabs.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveTab(id)}
              className={`tab-item ${activeTab === id ? "active" : ""}`}
            >
              <Icon size={14} />
              {label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-5 py-4">
        <div className="card max-w-2xl mx-auto">
          {activeTab === "audio" && (
            <AudioSettings config={config} onSave={saveConfig} isCapturing={isCapturing} />
          )}
          {activeTab === "whisper" && <WhisperSettings config={config} onSave={saveConfig} />}
          {activeTab === "theme" && <ThemeSettings config={config} onSave={saveConfig} />}
          {activeTab === "shortcuts" && <ShortcutsSettings config={config} onSave={saveConfig} />}
        </div>
      </div>
    </div>
  );
}
