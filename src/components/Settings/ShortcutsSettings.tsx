import { useState } from "react";
import { AppConfig } from "../../hooks/useSettings";
import { Check, Keyboard } from "lucide-react";

interface ShortcutsSettingsProps {
  config: AppConfig;
  onSave: (config: AppConfig) => Promise<void>;
}

export function ShortcutsSettings({ config, onSave }: ShortcutsSettingsProps) {
  const [shortcuts, setShortcuts] = useState(config.shortcuts);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    try {
      await onSave({
        ...config,
        shortcuts,
      });
      setSaved(true);
      setError(null);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to save shortcuts";
      setError(msg);
    }
  };

  const handleShortcutChange = (key: keyof typeof shortcuts, value: string) => {
    setShortcuts({ ...shortcuts, [key]: value });
  };

  return (
    <div className="flex flex-col">
      <div className="section">
        <div className="section-title">Keyboard Shortcuts</div>

        {error && (
          <div className="mb-4 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
            {error}
          </div>
        )}

        <div className="space-y-4">
          <div className="flex items-center gap-3 p-3 bg-bg-base rounded-lg border border-border-subtle">
            <Keyboard size={16} className="text-text-muted flex-shrink-0" />
            <div className="flex-1">
              <div className="text-[13px] text-text-primary">Toggle Capture</div>
              <div className="text-[11px] text-text-muted">Start/stop audio capture</div>
            </div>
            <input
              type="text"
              value={shortcuts.toggle_capture}
              onChange={(e) => handleShortcutChange("toggle_capture", e.target.value)}
              className="input font-mono text-[12px] w-32"
              placeholder="Ctrl+Shift+S"
            />
          </div>

          <div className="flex items-center gap-3 p-3 bg-bg-base rounded-lg border border-border-subtle">
            <Keyboard size={16} className="text-text-muted flex-shrink-0" />
            <div className="flex-1">
              <div className="text-[13px] text-text-primary">Toggle Overlay</div>
              <div className="text-[11px] text-text-muted">Show/hide subtitle overlay</div>
            </div>
            <input
              type="text"
              value={shortcuts.toggle_overlay}
              onChange={(e) => handleShortcutChange("toggle_overlay", e.target.value)}
              className="input font-mono text-[12px] w-32"
              placeholder="Ctrl+Shift+O"
            />
          </div>

          <div className="flex items-center gap-3 p-3 bg-bg-base rounded-lg border border-border-subtle">
            <Keyboard size={16} className="text-text-muted flex-shrink-0" />
            <div className="flex-1">
              <div className="text-[13px] text-text-primary">Toggle Translation</div>
              <div className="text-[11px] text-text-muted">Enable/disable translation</div>
            </div>
            <input
              type="text"
              value={shortcuts.toggle_translation}
              onChange={(e) => handleShortcutChange("toggle_translation", e.target.value)}
              className="input font-mono text-[12px] w-32"
              placeholder="Ctrl+Shift+T"
            />
          </div>

          <div className="flex items-center gap-3 p-3 bg-bg-base rounded-lg border border-border-subtle">
            <Keyboard size={16} className="text-text-muted flex-shrink-0" />
            <div className="flex-1">
              <div className="text-[13px] text-text-primary">Clear History</div>
              <div className="text-[11px] text-text-muted">Clear transcription history</div>
            </div>
            <input
              type="text"
              value={shortcuts.clear_history}
              onChange={(e) => handleShortcutChange("clear_history", e.target.value)}
              className="input font-mono text-[12px] w-32"
              placeholder="Ctrl+Shift+H"
            />
          </div>
        </div>

        <div className="mt-4 p-3 bg-bg-surface rounded-lg border border-border-subtle">
          <p className="text-[12px] text-text-muted leading-relaxed">
            <strong className="text-text-secondary">Note:</strong> Shortcuts are global and work
            even when the app isn't focused. Restart the app after changing shortcuts.
          </p>
        </div>
      </div>

      <div className="section flex justify-end">
        <button onClick={handleSave} className="btn btn-primary btn-sm gap-2">
          {saved ? <Check size={14} /> : null}
          {saved ? "Saved" : "Save"}
        </button>
      </div>
    </div>
  );
}
