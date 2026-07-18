import { useState } from "react";
import { Download, FileText, FileVideo, FileCode, FileJson, Subtitles } from "lucide-react";

interface ExportMenuProps {
  transcriptionId: number;
  onExport: (id: number, format: string) => Promise<void>;
}

const FORMAT_OPTIONS = [
  { value: "srt", label: "SubRip (.srt)", icon: FileText, ext: ".srt" },
  { value: "vtt", label: "WebVTT (.vtt)", icon: FileVideo, ext: ".vtt" },
  { value: "ass", label: "Advanced SSA (.ass)", icon: Subtitles, ext: ".ass" },
  { value: "txt", label: "Plain Text (.txt)", icon: FileCode, ext: ".txt" },
  { value: "json", label: "JSON (.json)", icon: FileJson, ext: ".json" },
];

export function ExportMenu({ transcriptionId, onExport }: ExportMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [exporting, setExporting] = useState<string | null>(null);

  const handleExport = async (format: string) => {
    try {
      setExporting(format);
      await onExport(transcriptionId, format);
    } catch (err) {
      console.error("Export failed:", err);
    } finally {
      setExporting(null);
      setIsOpen(false);
    }
  };

  return (
    <div className="relative">
      <button onClick={() => setIsOpen(!isOpen)} className="btn btn-ghost btn-sm gap-2">
        <Download size={14} />
        Export
      </button>

      {isOpen && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setIsOpen(false)} />
          <div className="absolute right-0 top-full mt-1 z-50 card p-1 w-56 shadow-lg">
            {FORMAT_OPTIONS.map((opt) => {
              const Icon = opt.icon;
              return (
                <button
                  key={opt.value}
                  onClick={() => handleExport(opt.value)}
                  disabled={exporting !== null}
                  className="w-full flex items-center gap-2 px-3 py-2 text-[13px] text-text-secondary hover:bg-bg-hover rounded-md transition-colors"
                >
                  <Icon size={14} className="text-text-muted" />
                  <span>{opt.label}</span>
                  {exporting === opt.value && <span className="ml-auto animate-spin">...</span>}
                </button>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
