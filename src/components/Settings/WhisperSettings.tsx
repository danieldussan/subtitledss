import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../../hooks/useSettings";
import { Check, Zap, Loader2 } from "lucide-react";

interface WhisperSettingsProps {
  config: AppConfig;
  onSave: (config: AppConfig) => Promise<void>;
  loadedModel: string | null;
}

export function WhisperSettings({ config, onSave, loadedModel }: WhisperSettingsProps) {
  const [model, setModel] = useState(config.whisper.model);
  const [language, setLanguage] = useState(config.whisper.language);
  const [threads, setThreads] = useState(config.whisper.threads);
  const [gpu, setGpu] = useState(config.whisper.gpu);
  const [saved, setSaved] = useState(false);
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    try {
      setSwitching(true);
      setError(null);

      await onSave({
        ...config,
        whisper: { ...config.whisper, model, language, threads, gpu },
      });

      if (model !== config.whisper.model) {
        await invoke("switch_model", { modelName: model });
      }

      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to save whisper settings";
      setError(msg);
    } finally {
      setSwitching(false);
    }
  };

  const models = [
    { value: "tiny", label: "Tiny", size: "39 MB", desc: "Fastest, lowest accuracy" },
    { value: "base", label: "Base", size: "142 MB", desc: "Good balance for most uses" },
    { value: "small", label: "Small", size: "466 MB", desc: "Better accuracy" },
    { value: "medium", label: "Medium", size: "1.5 GB", desc: "High accuracy" },
    { value: "large-v3", label: "Large v3", size: "3.1 GB", desc: "Best accuracy" },
  ];

  const languages = [
    { value: "auto", label: "Auto-detect" },
    { value: "en", label: "English" },
    { value: "es", label: "Espa\u00f1ol" },
    { value: "fr", label: "Fran\u00e7ais" },
    { value: "de", label: "Deutsch" },
    { value: "it", label: "Italiano" },
    { value: "pt", label: "Portugu\u00eas" },
    { value: "ja", label: "\u65e5\u672c\u8a9e" },
    { value: "zh", label: "\u4e2d\u6587" },
    { value: "ko", label: "\ud55c\uad6d\uc5b4" },
    { value: "ar", label: "\u0627\u0644\u0639\u0631\u0628\u064a\u0629" },
    { value: "ru", label: "\u0420\u0443\u0441\u0441\u043a\u0438\u0439" },
    { value: "hi", label: "\u0939\u093f\u0928\u094d\u0926\u0940" },
  ];

  const cpuCount = navigator.hardwareConcurrency || 4;

  return (
    <div className="flex flex-col">
      {/* Model Selection */}
      <div className="section">
        <div className="section-title">Model</div>

        {error && (
          <div className="mb-4 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
            {error}
          </div>
        )}

        <div className="space-y-2">
          {models.map((m) => {
            const isLoaded = loadedModel === m.value;
            return (
              <button
                key={m.value}
                onClick={() => setModel(m.value)}
                className={`w-full flex items-center gap-3 p-3 rounded-lg text-left transition-all ${
                  model === m.value
                    ? "bg-accent-subtle border border-accent/30"
                    : "bg-bg-base border border-border-subtle hover:border-border-default"
                }`}
              >
                <div
                  className={`w-4 h-4 rounded-full border-2 flex items-center justify-center flex-shrink-0 ${
                    model === m.value ? "border-accent" : "border-border-strong"
                  }`}
                >
                  {model === m.value && <div className="w-2 h-2 rounded-full bg-accent" />}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-[13px] font-medium text-text-primary">{m.label}</span>
                    <span className="text-[11px] text-text-muted font-mono">{m.size}</span>
                    {isLoaded && (
                      <span className="text-[10px] font-medium uppercase text-success bg-success-subtle px-1.5 py-0.5 rounded">
                        Loaded
                      </span>
                    )}
                  </div>
                  <span className="text-[12px] text-text-secondary">{m.desc}</span>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Language */}
      <div className="section">
        <div className="section-title">Language</div>
        <select value={language} onChange={(e) => setLanguage(e.target.value)} className="select">
          {languages.map((l) => (
            <option key={l.value} value={l.value}>
              {l.label}
            </option>
          ))}
        </select>
      </div>

      {/* Performance */}
      <div className="section">
        <div className="section-title">Performance</div>
        <div className="space-y-4">
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="label mb-0">CPU Threads</label>
              <span className="text-[13px] font-mono text-text-primary">{threads}</span>
            </div>
            <input
              type="range"
              min="1"
              max={Math.min(cpuCount, 16)}
              value={threads}
              onChange={(e) => setThreads(parseInt(e.target.value))}
            />
            <div className="flex justify-between text-[11px] text-text-muted mt-1">
              <span>1</span>
              <span>{Math.min(cpuCount, 16)}</span>
            </div>
          </div>

          <div className="flex items-center justify-between p-3 bg-bg-base rounded-lg border border-border-subtle">
            <div className="flex items-center gap-2">
              <Zap size={14} className="text-text-muted" />
              <div>
                <span className="text-[13px] text-text-primary">GPU Acceleration</span>
                <p className="text-[11px] text-text-muted">CUDA / Vulkan / Metal</p>
              </div>
            </div>
            <div
              className={`toggle-switch ${gpu ? "active" : ""}`}
              onClick={() => setGpu(!gpu)}
              role="switch"
              aria-checked={gpu}
            />
          </div>
        </div>
      </div>

      {/* Save */}
      <div className="section flex justify-end">
        <button onClick={handleSave} disabled={switching} className="btn btn-primary btn-sm gap-2">
          {switching ? (
            <Loader2 size={14} className="animate-spin" />
          ) : saved ? (
            <Check size={14} />
          ) : null}
          {switching ? "Loading model..." : saved ? "Saved" : "Save"}
        </button>
      </div>
    </div>
  );
}
