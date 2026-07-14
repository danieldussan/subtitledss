import { useState } from "react";
import { AppConfig } from "../../hooks/useSettings";
import { Check, Languages, Download, Loader2, Trash2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface TranslationSettingsProps {
  config: AppConfig;
  onSave: (config: AppConfig) => Promise<void>;
}

export function TranslationSettings({ config, onSave }: TranslationSettingsProps) {
  const [enabled, setEnabled] = useState(config.translation.enabled);
  const [sourceLang, setSourceLang] = useState(config.translation.source_lang);
  const [targetLang, setTargetLang] = useState(config.translation.target_lang);
  const [showOriginal, setShowOriginal] = useState(config.translation.show_original);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Marian model state
  const [marianDownloaded, setMarianDownloaded] = useState<Record<string, boolean>>({});
  const [marianDownloading, setMarianDownloading] = useState<string | null>(null);
  const [marianDeleting, setMarianDeleting] = useState<string | null>(null);

  const supportedPairs = [
    { source: "en", target: "es", label: "English → Español" },
    { source: "es", target: "en", label: "Español → English" },
  ];

  const languages = [
    { value: "en", label: "English" },
    { value: "es", label: "Español" },
  ];

  const checkMarianModel = async (src: string, tgt: string) => {
    try {
      const key = `${src}-${tgt}`;
      const result = await invoke<boolean>("check_marian_model", {
        sourceLang: src,
        targetLang: tgt,
      });
      setMarianDownloaded((prev) => ({ ...prev, [key]: result }));
      return result;
    } catch {
      return false;
    }
  };

  const handleDownloadMarian = async (src: string, tgt: string) => {
    const key = `${src}-${tgt}`;
    try {
      setMarianDownloading(key);
      setError(null);
      await invoke("download_marian_model", { sourceLang: src, targetLang: tgt });
      setMarianDownloaded((prev) => ({ ...prev, [key]: true }));
    } catch (err) {
      const msg = typeof err === "string" ? err : "Download failed";
      setError(msg);
    } finally {
      setMarianDownloading(null);
    }
  };

  const handleDeleteMarian = async (src: string, tgt: string) => {
    const key = `${src}-${tgt}`;
    if (!confirm(`Delete Marian model ${src}→${tgt}?`)) return;
    try {
      setMarianDeleting(key);
      setError(null);
      await invoke("delete_marian_model", { sourceLang: src, targetLang: tgt });
      setMarianDownloaded((prev) => ({ ...prev, [key]: false }));
    } catch (err) {
      const msg = typeof err === "string" ? err : "Delete failed";
      setError(msg);
    } finally {
      setMarianDeleting(null);
    }
  };

  // Check model status on mount
  useState(() => {
    supportedPairs.forEach((pair) => {
      checkMarianModel(pair.source, pair.target);
    });
  });

  const handleSave = async () => {
    try {
      await onSave({
        ...config,
        translation: {
          ...config.translation,
          enabled,
          source_lang: sourceLang,
          target_lang: targetLang,
          show_original: showOriginal,
        },
      });
      setSaved(true);
      setError(null);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to save translation settings";
      setError(msg);
    }
  };

  return (
    <div className="flex flex-col">
      {/* Enable/Disable */}
      <div className="section">
        <div className="section-title">Translation</div>

        {error && (
          <div className="mb-4 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
            {error}
          </div>
        )}

        <div className="flex items-center justify-between p-3 bg-bg-base rounded-lg border border-border-subtle">
          <div className="flex items-center gap-2">
            <Languages size={14} className="text-text-muted" />
            <div>
              <span className="text-[13px] text-text-primary">Enable Translation</span>
              <p className="text-[11px] text-text-muted">
                Offline translation via Marian MT (100% local, no server needed)
              </p>
            </div>
          </div>
          <div
            className={`toggle-switch ${enabled ? "active" : ""}`}
            onClick={() => setEnabled(!enabled)}
            role="switch"
            aria-checked={enabled}
          />
        </div>
      </div>

      {/* Languages */}
      {enabled && (
        <div className="section">
          <div className="section-title">Languages</div>
          <div className="space-y-3">
            <div>
              <label className="label mb-1">Source Language</label>
              <select
                value={sourceLang}
                onChange={(e) => setSourceLang(e.target.value)}
                className="select"
              >
                {languages.map((l) => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="label mb-1">Target Language</label>
              <select
                value={targetLang}
                onChange={(e) => setTargetLang(e.target.value)}
                className="select"
              >
                {languages.map((l) => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>
      )}

      {/* Marian Model Download */}
      {enabled && (
        <div className="section">
          <div className="section-title">Translation Model</div>
          <div className="space-y-2">
            {supportedPairs.map((pair) => {
              const key = `${pair.source}-${pair.target}`;
              const isDownloaded = marianDownloaded[key] ?? false;
              const isDownloading = marianDownloading === key;
              const isDeleting = marianDeleting === key;
              const isSelected = sourceLang === pair.source && targetLang === pair.target;

              return (
                <div
                  key={key}
                  className={`flex items-center justify-between p-3 rounded-lg border transition-all ${
                    isSelected
                      ? "bg-accent-subtle border-accent/30"
                      : "bg-bg-base border-border-subtle"
                  }`}
                >
                  <div className="flex-1 min-w-0">
                    <span className="text-[13px] font-medium text-text-primary">{pair.label}</span>
                    <p className="text-[11px] text-text-muted">
                      {isDownloaded ? "~300 MB" : "Not downloaded"}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    {isDownloaded ? (
                      <>
                        <span className="text-[10px] font-medium text-success">Ready</span>
                        <button
                          onClick={() => handleDeleteMarian(pair.source, pair.target)}
                          disabled={isDeleting}
                          className="btn btn-ghost btn-sm gap-1 text-danger"
                          title="Delete model"
                        >
                          <Trash2 size={12} />
                        </button>
                      </>
                    ) : isDownloading ? (
                      <div className="flex items-center gap-2">
                        <Loader2 size={14} className="animate-spin text-accent" />
                        <span className="text-[11px] text-text-muted">Downloading...</span>
                      </div>
                    ) : (
                      <button
                        onClick={() => handleDownloadMarian(pair.source, pair.target)}
                        className="btn btn-primary btn-sm gap-1.5"
                      >
                        <Download size={12} />
                        Download
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
          <p className="text-[11px] text-text-muted mt-2">
            Models run locally on CPU via candle. No data leaves your machine.
          </p>
        </div>
      )}

      {/* Display Options */}
      {enabled && (
        <div className="section">
          <div className="section-title">Display</div>
          <div className="flex items-center justify-between p-3 bg-bg-base rounded-lg border border-border-subtle">
            <div>
              <span className="text-[13px] text-text-primary">Show Original Text</span>
              <p className="text-[11px] text-text-muted">Display original text below translation</p>
            </div>
            <div
              className={`toggle-switch ${showOriginal ? "active" : ""}`}
              onClick={() => setShowOriginal(!showOriginal)}
              role="switch"
              aria-checked={showOriginal}
            />
          </div>
        </div>
      )}

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
