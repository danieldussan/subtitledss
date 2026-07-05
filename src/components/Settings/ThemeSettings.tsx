import { useState } from "react";
import { AppConfig } from "../../hooks/useSettings";
import { Check, Eye, EyeOff } from "lucide-react";

interface ThemeSettingsProps {
  config: AppConfig;
  onSave: (config: AppConfig) => Promise<void>;
}

export function ThemeSettings({ config, onSave }: ThemeSettingsProps) {
  const [fontSize, setFontSize] = useState(config.overlay.font_size);
  const [fontColor, setFontColor] = useState(config.overlay.font_color);
  const [bgColor, setBgColor] = useState(config.overlay.background_color);
  const [opacity, setOpacity] = useState(config.overlay.opacity);
  const [autoHide, setAutoHide] = useState(config.overlay.auto_hide);
  const [autoHideDelay, setAutoHideDelay] = useState(config.overlay.auto_hide_delay);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    try {
      await onSave({
        ...config,
        overlay: {
          ...config.overlay,
          font_size: fontSize,
          font_color: fontColor,
          background_color: bgColor,
          opacity,
          auto_hide: autoHide,
          auto_hide_delay: autoHideDelay,
        },
      });
      setSaved(true);
      setError(null);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to save overlay settings";
      setError(msg);
    }
  };

  return (
    <div className="flex flex-col">
      {/* Preview */}
      <div className="section">
        <div className="section-title">Preview</div>
        <div
          className="relative rounded-lg overflow-hidden flex items-center justify-center py-8"
          style={{ background: "#000" }}
        >
          <div
            className="px-6 py-3 rounded-lg text-center max-w-[90%]"
            style={{
              fontSize: `${fontSize}px`,
              color: fontColor,
              backgroundColor: bgColor,
              opacity,
            }}
          >
            This is a subtitle preview
          </div>
        </div>
      </div>

      {/* Typography */}
      <div className="section">
        <div className="section-title">Typography</div>

        {error && (
          <div className="mb-4 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
            {error}
          </div>
        )}

        <div className="space-y-4">
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="label mb-0">Font Size</label>
              <span className="text-[13px] font-mono text-text-primary">{fontSize}px</span>
            </div>
            <input
              type="range"
              min="12"
              max="72"
              value={fontSize}
              onChange={(e) => setFontSize(parseInt(e.target.value))}
            />
            <div className="flex justify-between text-[11px] text-text-muted mt-1">
              <span>12</span>
              <span>42</span>
              <span>72</span>
            </div>
          </div>
        </div>
      </div>

      {/* Colors */}
      <div className="section">
        <div className="section-title">Colors</div>
        <div className="space-y-4">
          <div>
            <label className="label">Font Color</label>
            <div className="flex items-center gap-3">
              <input
                type="color"
                value={fontColor}
                onChange={(e) => setFontColor(e.target.value)}
                className="w-10 h-10"
              />
              <input
                type="text"
                value={fontColor}
                onChange={(e) => setFontColor(e.target.value)}
                className="input font-mono text-[12px]"
              />
            </div>
          </div>

          <div>
            <label className="label">Background Color</label>
            <div className="flex items-center gap-3">
              <input
                type="color"
                value={bgColor.slice(0, 7)}
                onChange={(e) => setBgColor(e.target.value + bgColor.slice(7))}
                className="w-10 h-10"
              />
              <input
                type="text"
                value={bgColor}
                onChange={(e) => setBgColor(e.target.value)}
                className="input font-mono text-[12px]"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Behavior */}
      <div className="section">
        <div className="section-title">Behavior</div>
        <div className="space-y-4">
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="label mb-0">Opacity</label>
              <span className="text-[13px] font-mono text-text-primary">
                {Math.round(opacity * 100)}%
              </span>
            </div>
            <input
              type="range"
              min="10"
              max="100"
              value={Math.round(opacity * 100)}
              onChange={(e) => setOpacity(parseInt(e.target.value) / 100)}
            />
          </div>

          <div className="flex items-center justify-between p-3 bg-bg-base rounded-lg border border-border-subtle">
            <div className="flex items-center gap-2">
              {autoHide ? (
                <Eye size={14} className="text-text-muted" />
              ) : (
                <EyeOff size={14} className="text-text-muted" />
              )}
              <div>
                <span className="text-[13px] text-text-primary">Auto-hide</span>
                <p className="text-[11px] text-text-muted">Hide overlay when no speech detected</p>
              </div>
            </div>
            <div
              className={`toggle-switch ${autoHide ? "active" : ""}`}
              onClick={() => setAutoHide(!autoHide)}
              role="switch"
              aria-checked={autoHide}
            />
          </div>

          {autoHide && (
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="label mb-0">Delay</label>
                <span className="text-[13px] font-mono text-text-primary">
                  {(autoHideDelay / 1000).toFixed(0)}s
                </span>
              </div>
              <input
                type="range"
                min="1000"
                max="10000"
                step="1000"
                value={autoHideDelay}
                onChange={(e) => setAutoHideDelay(parseInt(e.target.value))}
              />
              <div className="flex justify-between text-[11px] text-text-muted mt-1">
                <span>1s</span>
                <span>5s</span>
                <span>10s</span>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Save */}
      <div className="section flex justify-end">
        <button onClick={handleSave} className="btn btn-primary btn-sm gap-2">
          {saved ? <Check size={14} /> : null}
          {saved ? "Saved" : "Save"}
        </button>
      </div>
    </div>
  );
}
