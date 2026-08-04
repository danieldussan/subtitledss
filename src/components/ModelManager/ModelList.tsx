import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Download,
  Trash2,
  Check,
  Loader2,
  HardDrive,
  Zap,
  Gauge,
  Play,
  Sparkles,
} from "lucide-react";

interface AvailableModel {
  name: string;
  label: string;
  engine: string;
  size_mb: number;
  languages: string;
  description: string;
  downloaded: boolean;
}

interface ModelListProps {
  onModelSwitched?: (modelName: string) => void;
}

function formatSize(mb: number) {
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${Math.round(mb)} MB`;
}

function getModelTier(model: AvailableModel) {
  if (model.engine === "sherpa") {
    return model.name === "canary-1b-v2-int8"
      ? { label: "Precise", icon: HardDrive, color: "text-warning" }
      : { label: "Fast", icon: Zap, color: "text-success" };
  }
  if (model.size_mb < 100) return { label: "Fast", icon: Zap, color: "text-success" };
  if (model.size_mb < 500) return { label: "Balanced", icon: Gauge, color: "text-accent" };
  return { label: "Precise", icon: HardDrive, color: "text-warning" };
}

export function ModelList({ onModelSwitched }: ModelListProps) {
  const [models, setModels] = useState<AvailableModel[]>([]);
  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [loadingModels, setLoadingModels] = useState(true);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchModels();
    fetchLoadedModel();

    const unlisten = listen<{ model: string }>("model-changed", (event) => {
      setLoadedModel(event.payload.model);
      onModelSwitched?.(event.payload.model);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const fetchLoadedModel = async () => {
    try {
      const model = await invoke<string | null>("get_loaded_model");
      setLoadedModel(model);
    } catch (err) {
      console.error("Failed to fetch loaded model:", err);
    }
  };

  const fetchModels = async () => {
    try {
      setLoadingModels(true);
      const list = await invoke<AvailableModel[]>("list_available_models");
      setModels(list);
      setError(null);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to load model catalog";
      setError(msg);
      console.error("Failed to load model catalog:", err);
    } finally {
      setLoadingModels(false);
    }
  };

  const setModelDownloaded = (name: string, downloaded: boolean) => {
    setModels((prev) => prev.map((m) => (m.name === name ? { ...m, downloaded } : m)));
  };

  const handleDownload = async (modelName: string) => {
    try {
      setDownloading(modelName);
      setDownloadProgress(0);
      setError(null);
      const progressInterval = setInterval(() => {
        setDownloadProgress((p) => Math.min(p + Math.random() * 15, 90));
      }, 500);

      await invoke("download_model", { modelName });

      clearInterval(progressInterval);
      setDownloadProgress(100);
      setModelDownloaded(modelName, true);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Download failed";
      setError(msg);
      console.error("Download failed:", err);
    } finally {
      setDownloading(null);
      setDownloadProgress(0);
    }
  };

  const handleLoad = async (modelName: string) => {
    try {
      setLoading(modelName);
      setError(null);
      await invoke("switch_model", { modelName });
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to load model";
      setError(msg);
      console.error("Load failed:", err);
    } finally {
      setLoading(null);
    }
  };

  const handleDelete = async (modelName: string) => {
    if (confirm(`Delete model '${modelName}'? You can re-download it later.`)) {
      try {
        await invoke("delete_model", { modelName });
        setModelDownloaded(modelName, false);
        if (loadedModel === modelName) setLoadedModel(null);
        setError(null);
      } catch (err) {
        const msg = typeof err === "string" ? err : "Delete failed";
        setError(msg);
        console.error("Delete failed:", err);
      }
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="px-5 pt-4 pb-3">
        <h2 className="text-[15px] font-semibold text-text-primary">Model Manager</h2>
        <p className="text-[12px] text-text-muted mt-0.5">
          Download and manage transcription models
        </p>
      </div>

      {/* Error */}
      {error && (
        <div className="mx-5 mb-3 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
          {error}
        </div>
      )}

      {/* Models */}
      <div className="flex-1 overflow-y-auto px-5 pb-4">
        {loadingModels ? (
          <div className="flex items-center justify-center h-32 gap-2 text-text-muted">
            <Loader2 size={16} className="animate-spin" />
            <span className="text-[12px]">Loading models...</span>
          </div>
        ) : (
          <div className="space-y-2">
            {models.map((model) => {
              const isDownloaded = model.downloaded;
              const isDownloading = downloading === model.name;
              const isLoading = loading === model.name;
              const isLoaded = loadedModel === model.name;
              const tier = getModelTier(model);
              const TierIcon = tier.icon;
              const isRecommended = model.name === "parakeet-tdt-0.6b-v3-int8";

              return (
                <div key={model.name} className="card p-4">
                  <div className="flex items-start justify-between gap-4">
                    {/* Info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1 flex-wrap">
                        <span className="text-[14px] font-medium text-text-primary">
                          {model.label}
                        </span>
                        <span
                          className={`text-[10px] font-medium uppercase px-1.5 py-0.5 rounded ${
                            model.engine === "sherpa"
                              ? "bg-accent-subtle text-accent"
                              : "bg-bg-surface text-text-muted border border-border-subtle"
                          }`}
                        >
                          {model.engine}
                        </span>
                        <div className={`flex items-center gap-1 ${tier.color}`}>
                          <TierIcon size={11} />
                          <span className="text-[10px] font-medium uppercase">{tier.label}</span>
                        </div>
                        {isRecommended && (
                          <span className="text-[10px] font-medium uppercase text-warning bg-warning-subtle px-1.5 py-0.5 rounded flex items-center gap-1">
                            <Sparkles size={10} />
                            Recommended
                          </span>
                        )}
                        {isLoaded && (
                          <span className="text-[10px] font-medium uppercase text-success bg-success-subtle px-1.5 py-0.5 rounded">
                            Active
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-3 text-[12px] text-text-muted">
                        <span className="font-mono">{formatSize(model.size_mb)}</span>
                        <span className="text-border-strong">|</span>
                        <span className="truncate">{model.languages}</span>
                      </div>
                      <div className="text-[12px] text-text-secondary mt-1">
                        {model.description}
                      </div>
                    </div>

                    {/* Actions */}
                    <div className="flex items-center gap-2 flex-shrink-0">
                      {isDownloaded ? (
                        <>
                          <button
                            onClick={() => handleLoad(model.name)}
                            disabled={isLoading || isLoaded}
                            className={`btn btn-sm gap-1.5 ${
                              isLoaded ? "btn-ghost text-success" : "btn-primary"
                            }`}
                          >
                            {isLoading ? (
                              <Loader2 size={12} className="animate-spin" />
                            ) : isLoaded ? (
                              <Check size={12} />
                            ) : (
                              <Play size={12} />
                            )}
                            {isLoaded ? "Loaded" : "Load"}
                          </button>
                          <button
                            onClick={() => handleDelete(model.name)}
                            className="btn btn-ghost btn-sm gap-1 text-danger"
                            title="Delete model"
                          >
                            <Trash2 size={12} />
                          </button>
                        </>
                      ) : isDownloading ? (
                        <div className="flex items-center gap-2 min-w-[120px]">
                          <Loader2 size={14} className="animate-spin text-accent flex-shrink-0" />
                          <div className="flex-1">
                            <div className="h-1.5 bg-bg-base rounded-full overflow-hidden">
                              <div
                                className="h-full bg-accent rounded-full transition-all duration-300"
                                style={{ width: `${downloadProgress}%` }}
                              />
                            </div>
                            <span className="text-[10px] text-text-muted mt-0.5 block font-mono">
                              {Math.round(downloadProgress)}%
                            </span>
                          </div>
                        </div>
                      ) : (
                        <button
                          onClick={() => handleDownload(model.name)}
                          className="btn btn-primary btn-sm gap-1.5"
                        >
                          <Download size={12} />
                          Download
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Info */}
        <div className="mt-4 p-3 bg-bg-surface rounded-lg border border-border-subtle">
          <p className="text-[12px] text-text-muted leading-relaxed">
            <strong className="text-text-secondary">Tip:</strong> For real-time subtitles in
            Spanish, download and load{" "}
            <strong className="text-text-secondary">Parakeet TDT 0.6B v3</strong> — it is up to 50x
            faster than Whisper small with better accuracy.
          </p>
        </div>
      </div>
    </div>
  );
}
