import { useState } from "react";
import { useSettings } from "../../hooks/useSettings";
import { Check } from "lucide-react";
import { InlineControl } from "../Settings/InlineControl";

export function OverlayPage() {
  const { config, loading, error, saveConfig } = useSettings();

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        <div className="text-center">
          <div className="text-sm font-medium mb-1">Loading overlay settings...</div>
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
        <OverlaySettings config={config} onSave={saveConfig} />
      </div>
    </div>
  );
}

function OverlaySettings({
  config,
  onSave,
}: {
  config: ReturnType<typeof useSettings>["config"];
  onSave: ReturnType<typeof useSettings>["saveConfig"];
}) {
  if (!config) return null;

  const [fontSize, setFontSize] = useState(config.overlay.font_size);
  const [fontColor, setFontColor] = useState(config.overlay.font_color);
  const [bgColor, setBgColor] = useState(config.overlay.background_color);
  const [opacity, setOpacity] = useState(config.overlay.opacity);
  const [autoHide, setAutoHide] = useState(config.overlay.auto_hide);
  const [autoHideDelay, setAutoHideDelay] = useState(config.overlay.auto_hide_delay);
  const [displayDuration, setDisplayDuration] = useState(config.overlay.display_duration_ms);
  const [fadeDuration, setFadeDuration] = useState(config.overlay.fade_duration_ms);
  const [maxLines, setMaxLines] = useState(config.overlay.max_visible_lines);
  const [lineGap, setLineGap] = useState(config.overlay.line_gap);
  const [maxLineWidth, setMaxLineWidth] = useState(config.overlay.max_line_width);
  const [alwaysOnTop, setAlwaysOnTop] = useState(config.overlay.always_on_top);
  const [clickThrough, setClickThrough] = useState(config.overlay.click_through);
  const [overlayWidth, setOverlayWidth] = useState(config.overlay.width);
  const [overlayHeight, setOverlayHeight] = useState(config.overlay.height);
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
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
        display_duration_ms: displayDuration,
        fade_duration_ms: fadeDuration,
        max_visible_lines: maxLines,
        line_gap: lineGap,
        max_line_width: maxLineWidth,
        always_on_top: alwaysOnTop,
        click_through: clickThrough,
        width: overlayWidth,
        height: overlayHeight,
      },
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="flex flex-col">
      {/* Preview */}
      <div className="section">
        <div className="section-title">Preview</div>
        <div
          className="rounded-lg flex items-center justify-center py-8"
          style={{ background: "#000" }}
        >
          <div
            style={{
              fontSize: `${fontSize}px`,
              color: fontColor,
              backgroundColor: bgColor,
              opacity,
              padding: "8px 16px",
              borderRadius: "4px",
              maxWidth: "90%",
              textAlign: "center",
            }}
          >
            Sample subtitle text
          </div>
        </div>
      </div>

      {/* Appearance */}
      <div className="section">
        <div className="section-title">Appearance</div>
        <div className="space-y-4">
          <InlineControl label="Font Size" description={`${fontSize}px`}>
            <input
              type="range"
              min="12"
              max="72"
              value={fontSize}
              onChange={(e) => setFontSize(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>

          <InlineControl label="Font Color">
            <div className="flex items-center gap-2">
              <input
                type="color"
                value={fontColor}
                onChange={(e) => setFontColor(e.target.value)}
                className="w-8 h-8"
              />
              <span className="text-[12px] font-mono text-text-muted">{fontColor}</span>
            </div>
          </InlineControl>

          <InlineControl label="Background Color">
            <div className="flex items-center gap-2">
              <input
                type="color"
                value={bgColor.slice(0, 7)}
                onChange={(e) => setBgColor(e.target.value + bgColor.slice(7))}
                className="w-8 h-8"
              />
              <span className="text-[12px] font-mono text-text-muted">{bgColor}</span>
            </div>
          </InlineControl>

          <InlineControl label="Opacity" description={`${Math.round(opacity * 100)}%`}>
            <input
              type="range"
              min="0.1"
              max="1"
              step="0.05"
              value={opacity}
              onChange={(e) => setOpacity(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>
        </div>
      </div>

      {/* Position & Size */}
      <div className="section">
        <div className="section-title">Position & Size</div>
        <div className="space-y-4">
          <InlineControl label="Width" description={`${overlayWidth}px`}>
            <input
              type="range"
              min="200"
              max="1920"
              step="10"
              value={overlayWidth}
              onChange={(e) => setOverlayWidth(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>

          <InlineControl label="Height" description={`${overlayHeight}px`}>
            <input
              type="range"
              min="40"
              max="400"
              step="10"
              value={overlayHeight}
              onChange={(e) => setOverlayHeight(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>
        </div>
      </div>

      {/* Behavior */}
      <div className="section">
        <div className="section-title">Behavior</div>
        <div className="space-y-4">
          <InlineControl label="Always on Top" description="Keep overlay above other windows">
            <div
              className={`toggle-switch ${alwaysOnTop ? "active" : ""}`}
              onClick={() => setAlwaysOnTop(!alwaysOnTop)}
            />
          </InlineControl>

          <InlineControl label="Click Through" description="Allow mouse events to pass through">
            <div
              className={`toggle-switch ${clickThrough ? "active" : ""}`}
              onClick={() => setClickThrough(!clickThrough)}
            />
          </InlineControl>

          <InlineControl label="Auto-hide" description="Hide when no speech detected">
            <div
              className={`toggle-switch ${autoHide ? "active" : ""}`}
              onClick={() => setAutoHide(!autoHide)}
            />
          </InlineControl>

          {autoHide && (
            <InlineControl label="Auto-hide Delay" description={`${autoHideDelay / 1000}s`}>
              <input
                type="range"
                min="1000"
                max="30000"
                step="1000"
                value={autoHideDelay}
                onChange={(e) => setAutoHideDelay(Number(e.target.value))}
                className="w-32"
              />
            </InlineControl>
          )}
        </div>
      </div>

      {/* Timing */}
      <div className="section">
        <div className="section-title">Timing</div>
        <div className="space-y-4">
          <InlineControl label="Display Duration" description={`${displayDuration / 1000}s`}>
            <input
              type="range"
              min="2000"
              max="60000"
              step="1000"
              value={displayDuration}
              onChange={(e) => setDisplayDuration(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>

          <InlineControl label="Fade Duration" description={`${fadeDuration / 1000}s`}>
            <input
              type="range"
              min="500"
              max="10000"
              step="500"
              value={fadeDuration}
              onChange={(e) => setFadeDuration(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>
        </div>
      </div>

      {/* Text Layout */}
      <div className="section">
        <div className="section-title">Text Layout</div>
        <div className="space-y-4">
          <InlineControl label="Max Visible Lines" description={`${maxLines} lines`}>
            <input
              type="range"
              min="1"
              max="10"
              value={maxLines}
              onChange={(e) => setMaxLines(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>

          <InlineControl label="Line Gap" description={`${lineGap}px`}>
            <input
              type="range"
              min="0"
              max="20"
              value={lineGap}
              onChange={(e) => setLineGap(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>

          <InlineControl label="Max Line Width" description={`${maxLineWidth} chars`}>
            <input
              type="range"
              min="20"
              max="200"
              value={maxLineWidth}
              onChange={(e) => setMaxLineWidth(Number(e.target.value))}
              className="w-32"
            />
          </InlineControl>
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
