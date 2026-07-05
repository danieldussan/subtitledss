import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Download, Trash2, Check, Loader2, HardDrive, Zap, Gauge, Play } from "lucide-react";

interface ModelInfo {
  name: string;
  filename: string;
  url: string;
  size_mb: number;
  sha256: string;
}

export function ModelList() {
  const [models] = useState<ModelInfo[]>([
    { name: "tiny", filename: "ggml-tiny.bin", url: "", size_mb: 39, sha256: "" },
    { name: "base", filename: "ggml-base.bin", url: "", size_mb: 142, sha256: "" },
    { name: "small", filename: "ggml-small.bin", url: "", size_mb: 466, sha256: "" },
    { name: "medium", filename: "ggml-medium.bin", url: "", size_mb: 1500, sha256: "" },
    { name: "large-v3", filename: "ggml-large-v3.bin", url: "", size_mb: 3100, sha256: "" },
  ]);
  const [downloaded, setDownloaded] = useState<string[]>([]);
  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkDownloadedModels();
  }, []);

  const checkDownloadedModels = async () => {
    try {
      const result = await invoke<string[]>("list_downloaded_models");
      const names = result
        .map((f) => {
          const match = f.match(/ggml-(.+)\.bin/);
          return match ? match[1] : null;
        })
        .filter(Boolean) as string[];
      setDownloaded(names);
      setError(null);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to check models";
      setError(msg);
      console.error("Failed to check models:", err);
    }
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
      setDownloaded([...downloaded, modelName]);
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
      await invoke("load_model", { modelName });
      setLoadedModel(modelName);
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
        setDownloaded(downloaded.filter((m) => m !== modelName));
        if (loadedModel === modelName) setLoadedModel(null);
        setError(null);
      } catch (err) {
        const msg = typeof err === "string" ? err : "Delete failed";
        setError(msg);
        console.error("Delete failed:", err);
      }
    }
  };

  const getModelTier = (sizeMb: number) => {
    if (sizeMb < 100) return { label: "Fast", icon: Zap, color: "text-success" };
    if (sizeMb < 500) return { label: "Balanced", icon: Gauge, color: "text-accent" };
    return { label: "Precise", icon: HardDrive, color: "text-warning" };
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="px-5 pt-4 pb-3">
        <h2 className="text-[15px] font-semibold text-text-primary">Model Manager</h2>
        <p className="text-[12px] text-text-muted mt-0.5">Download and manage Whisper models</p>
      </div>

      {/* Error */}
      {error && (
        <div className="mx-5 mb-3 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
          {error}
        </div>
      )}

      {/* Models */}
      <div className="flex-1 overflow-y-auto px-5 pb-4">
        <div className="space-y-2">
          {models.map((model) => {
            const isDownloaded = downloaded.includes(model.name);
            const isDownloading = downloading === model.name;
            const isLoading = loading === model.name;
            const isLoaded = loadedModel === model.name;
            const tier = getModelTier(model.size_mb);
            const TierIcon = tier.icon;

            return (
              <div key={model.name} className="card p-4">
                <div className="flex items-start justify-between gap-4">
                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-[14px] font-medium text-text-primary capitalize">
                        {model.name}
                      </span>
                      <div className={`flex items-center gap-1 ${tier.color}`}>
                        <TierIcon size={11} />
                        <span className="text-[10px] font-medium uppercase">{tier.label}</span>
                      </div>
                      {isLoaded && (
                        <span className="text-[10px] font-medium uppercase text-success bg-success-subtle px-1.5 py-0.5 rounded">
                          Active
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-3 text-[12px] text-text-muted">
                      <span className="font-mono">{model.size_mb} MB</span>
                      <span className="text-border-strong">|</span>
                      <span className="font-mono">{model.filename}</span>
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

        {/* Info */}
        <div className="mt-4 p-3 bg-bg-surface rounded-lg border border-border-subtle">
          <p className="text-[12px] text-text-muted leading-relaxed">
            <strong className="text-text-secondary">Tip:</strong> Download a model, then click Load
            to activate it. Start with Tiny or Base for real-time transcription.
          </p>
        </div>
      </div>
    </div>
  );
}
