import { useSettings } from "../../hooks/useSettings";
import { ShortcutsSettings } from "../Settings/ShortcutsSettings";

export function ShortcutsPage() {
  const { config, loading, error, saveConfig } = useSettings();

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        <div className="text-center">
          <div className="text-sm font-medium mb-1">Loading shortcuts...</div>
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
        <ShortcutsSettings config={config} onSave={saveConfig} />
      </div>
    </div>
  );
}
