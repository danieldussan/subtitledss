import { useSettings } from "../../hooks/useSettings";
import { SettingsLayout } from "./SettingsLayout";
import { Loader2 } from "lucide-react";

interface SettingsPanelProps {
  isCapturing: boolean;
}

export function SettingsPanel({ isCapturing }: SettingsPanelProps) {
  const { config, loading, error, saveConfig } = useSettings();

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

  return (
    <SettingsLayout
      config={config}
      onSave={saveConfig}
      isCapturing={isCapturing}
    />
  );
}
