import { useSettings } from "../../hooks/useSettings";
import { AudioSettings } from "../Settings/AudioSettings";

export function AudioPage() {
  const { config, loading, error, saveConfig } = useSettings();

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        <div className="text-center">
          <div className="text-sm font-medium mb-1">Loading audio settings...</div>
        </div>
      </div>
    );
  }

  if (error || !config) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        <div className="text-center">
          <div className="text-sm font-medium mb-1 text-danger">Error loading settings</div>
          <div className="text-xs">{error || "No config available"}</div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="max-w-3xl mx-auto w-full">
        <AudioSettings config={config} onSave={saveConfig} isCapturing={false} />
      </div>
    </div>
  );
}
