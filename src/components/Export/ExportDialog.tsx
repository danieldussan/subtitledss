import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { X, Download, FileText, FileVideo, FileCode, FileJson } from "lucide-react";

interface ExportEntry {
  id: number;
  timestamp: string;
  language: string;
  original_text: string;
  translation: string | null;
}

interface ExportDialogProps {
  entries: ExportEntry[];
  onClose: () => void;
}

type ExportFormat = "srt" | "vtt" | "txt" | "json";

const FORMAT_OPTIONS: {
  value: ExportFormat;
  label: string;
  icon: React.ElementType;
  ext: string;
}[] = [
  { value: "srt", label: "SubRip (.srt)", icon: FileText, ext: ".srt" },
  { value: "vtt", label: "WebVTT (.vtt)", icon: FileVideo, ext: ".vtt" },
  { value: "txt", label: "Plain Text (.txt)", icon: FileCode, ext: ".txt" },
  { value: "json", label: "JSON (.json)", icon: FileJson, ext: ".json" },
];

export function ExportDialog({ entries, onClose }: ExportDialogProps) {
  const [format, setFormat] = useState<ExportFormat>("srt");
  const [exporting, setExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleExport = async () => {
    try {
      setExporting(true);
      setError(null);

      const selectedFormat = FORMAT_OPTIONS.find((f) => f.value === format);
      const path = await save({
        defaultPath: `transcription${selectedFormat?.ext || ".srt"}`,
        filters: [
          {
            name: selectedFormat?.label || "All Files",
            extensions: [format],
          },
        ],
      });

      if (path) {
        await invoke("export_history", {
          entries,
          format,
          path,
        });
        onClose();
      }
    } catch (err) {
      const msg = typeof err === "string" ? err : "Export failed";
      setError(msg);
      console.error("Export failed:", err);
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="card w-full max-w-md mx-4 p-0 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-border-subtle">
          <div className="flex items-center gap-2">
            <Download size={18} className="text-accent" />
            <h3 className="text-[15px] font-semibold text-text-primary">Export Transcription</h3>
          </div>
          <button onClick={onClose} className="btn btn-ghost btn-sm p-1">
            <X size={16} />
          </button>
        </div>

        {/* Content */}
        <div className="px-5 py-4 space-y-4">
          <p className="text-[13px] text-text-secondary">
            Export {entries.length} transcription{entries.length !== 1 ? "s" : ""} as:
          </p>

          {error && (
            <div className="px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
              {error}
            </div>
          )}

          <div className="space-y-2">
            {FORMAT_OPTIONS.map((opt) => {
              const Icon = opt.icon;
              return (
                <label
                  key={opt.value}
                  className={`flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                    format === opt.value
                      ? "border-accent bg-accent/10"
                      : "border-border-subtle hover:border-border-default"
                  }`}
                >
                  <input
                    type="radio"
                    name="format"
                    value={opt.value}
                    checked={format === opt.value}
                    onChange={(e) => setFormat(e.target.value as ExportFormat)}
                    className="sr-only"
                  />
                  <Icon
                    size={18}
                    className={format === opt.value ? "text-accent" : "text-text-muted"}
                  />
                  <span
                    className={`text-[13px] ${
                      format === opt.value ? "text-text-primary" : "text-text-secondary"
                    }`}
                  >
                    {opt.label}
                  </span>
                </label>
              );
            })}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-border-subtle bg-bg-surface/50">
          <button onClick={onClose} className="btn btn-ghost btn-sm">
            Cancel
          </button>
          <button
            onClick={handleExport}
            disabled={exporting}
            className="btn btn-primary btn-sm gap-2"
          >
            {exporting ? <span className="animate-spin">⏳</span> : <Download size={14} />}
            {exporting ? "Exporting..." : "Export"}
          </button>
        </div>
      </div>
    </div>
  );
}
