import { Upload, Film, FileVideo } from "lucide-react";

interface VideoPickerProps {
  selectedFile: string | null;
  onSelectFile: (path: string) => void;
  onTranscribe: () => void;
  disabled?: boolean;
  language?: string;
  onLanguageChange?: (lang: string) => void;
  targetLanguage?: string;
  onTargetLanguageChange?: (lang: string) => void;
  diarization?: boolean;
  onDiarizationChange?: (enabled: boolean) => void;
}

export function VideoPicker({
  selectedFile,
  onSelectFile,
  onTranscribe,
  disabled,
  language = "auto",
  onLanguageChange,
  targetLanguage = "none",
  onTargetLanguageChange,
  diarization = false,
  onDiarizationChange,
}: VideoPickerProps) {
  const handleFileSelect = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Video/Audio Files",
            extensions: ["mp4", "mkv", "avi", "webm", "mov", "mp3", "wav", "flac", "ogg"],
          },
        ],
      });
      if (selected && typeof selected === "string") {
        onSelectFile(selected);
      }
    } catch (err) {
      console.error("Failed to open file dialog:", err);
    }
  };

  const fileName = selectedFile?.split(/[/\\]/).pop() || "";

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
  ];

  return (
    <div className="card p-5">
      <div className="section-title mb-4">Select Video</div>

      <div className="flex items-center gap-4">
        <button onClick={handleFileSelect} className="btn btn-primary gap-2" disabled={disabled}>
          <Upload size={16} />
          Choose File
        </button>

        {selectedFile && (
          <div className="flex items-center gap-2 text-[13px] text-text-secondary">
            <FileVideo size={16} className="text-accent" />
            <span className="truncate max-w-[300px]">{fileName}</span>
          </div>
        )}
      </div>

      {selectedFile && (
        <div className="flex items-center gap-4 mt-4">
          <div className="flex items-center gap-2">
            <label className="text-[12px] text-text-muted">Language</label>
            <select
              value={language}
              onChange={(e) => onLanguageChange?.(e.target.value)}
              className="select select-sm"
              disabled={disabled}
            >
              {languages.map((l) => (
                <option key={l.value} value={l.value}>
                  {l.label}
                </option>
              ))}
            </select>
          </div>

          <div className="flex items-center gap-2">
            <label className="text-[12px] text-text-muted">Translate to</label>
            <select
              value={targetLanguage}
              onChange={(e) => onTargetLanguageChange?.(e.target.value)}
              className="select select-sm"
              disabled={disabled}
            >
              <option value="none">No translation</option>
              {languages
                .filter((l) => l.value !== "auto")
                .map((l) => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
            </select>
          </div>

          <div className="flex items-center gap-2">
            <label className="text-[12px] text-text-muted">Speakers</label>
            <div
              className={`toggle-switch ${diarization ? "active" : ""}`}
              onClick={() => onDiarizationChange?.(!diarization)}
              role="switch"
              aria-checked={diarization}
            />
          </div>

          <button
            onClick={onTranscribe}
            disabled={disabled}
            className="btn btn-primary btn-sm gap-2"
          >
            <Film size={14} />
            Transcribe
          </button>
        </div>
      )}
    </div>
  );
}
