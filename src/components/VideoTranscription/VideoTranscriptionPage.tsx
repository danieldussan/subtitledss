import { useState, useEffect } from "react";
import { Trash2, Clock, Film } from "lucide-react";
import { useVideoTranscription } from "../../hooks/useVideoTranscription";
import { VideoPicker } from "./VideoPicker";
import { ProgressIndicator } from "./ProgressIndicator";
import { TranscriptionViewer } from "./TranscriptionViewer";
import { ExportMenu } from "./ExportMenu";
import { AiPanel } from "./AiPanel";

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export function VideoTranscriptionPage() {
  const {
    selectedFile,
    setSelectedFile,
    result,
    step,
    progress,
    error,
    history,
    transcribe,
    deleteTranscription,
    exportTranscription,
    loadHistory,
  } = useVideoTranscription();

  const [language, setLanguage] = useState("auto");
  const [targetLanguage, setTargetLanguage] = useState("none");
  const [diarization, setDiarization] = useState(false);
  const [selectedHistoryId, setSelectedHistoryId] = useState<number | null>(null);

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const handleTranscribe = () => {
    if (selectedFile) {
      transcribe(
        selectedFile,
        language,
        diarization,
        targetLanguage === "none" ? undefined : targetLanguage,
      );
    }
  };

  const selectedEntry = selectedHistoryId ? history.find((h) => h.id === selectedHistoryId) : null;

  return (
    <div className="h-full p-5 space-y-5 overflow-y-auto">
      <div>
        <h1 className="text-lg font-semibold text-text-primary">Video Transcription</h1>
        <p className="text-[13px] text-text-muted mt-1">
          Transcribe video files with AI-powered speaker detection
        </p>
      </div>

      <VideoPicker
        selectedFile={selectedFile}
        onSelectFile={setSelectedFile}
        onTranscribe={handleTranscribe}
        disabled={step !== "idle" && step !== "done" && step !== "error"}
        language={language}
        onLanguageChange={setLanguage}
        targetLanguage={targetLanguage}
        onTargetLanguageChange={setTargetLanguage}
        diarization={diarization}
        onDiarizationChange={setDiarization}
      />

      {error && (
        <div className="card p-4 border-danger/20 bg-danger-subtle">
          <p className="text-[13px] text-danger">{error}</p>
        </div>
      )}

      <ProgressIndicator step={step} progress={progress} />

      {(result || selectedEntry) && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-5">
          <div className="lg:col-span-2 space-y-5">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-[15px] font-semibold text-text-primary">
                  {result?.video_name || selectedEntry?.video_name}
                </div>
                <div className="flex items-center gap-3 text-[12px] text-text-muted mt-1">
                  <span className="flex items-center gap-1">
                    <Clock size={12} />
                    {formatDuration(
                      result?.duration_seconds || selectedEntry?.duration_seconds || 0,
                    )}
                  </span>
                  <span>
                    {result?.segments?.length || selectedEntry?.segments?.length || 0} segments
                  </span>
                </div>
              </div>
              <ExportMenu
                transcriptionId={result?.id || selectedHistoryId || 0}
                onExport={exportTranscription}
              />
            </div>

            <TranscriptionViewer
              segments={result?.segments || selectedEntry?.segments || []}
              fullText={result?.full_text || selectedEntry?.full_text}
            />
          </div>

          <div className="lg:col-span-1">
            <AiPanel
              transcriptionId={result?.id || selectedHistoryId || 0}
              transcriptionText={result?.full_text || selectedEntry?.full_text || ""}
              language={selectedEntry?.language || language}
              targetLanguage={
                selectedEntry?.target_language ||
                (targetLanguage === "none" ? undefined : targetLanguage)
              }
              summary={selectedEntry?.summary || null}
            />
          </div>
        </div>
      )}

      {history.length > 0 && !result && !selectedEntry && (
        <div className="card p-5">
          <div className="section-title mb-4">Recent Transcriptions</div>
          <div className="space-y-2">
            {history.map((entry) => (
              <div
                key={entry.id}
                className="flex items-center justify-between p-3 rounded-lg bg-bg-surface hover:bg-bg-hover cursor-pointer transition-colors"
                onClick={() => setSelectedHistoryId(entry.id)}
              >
                <div className="flex items-center gap-3">
                  <Film size={16} className="text-accent" />
                  <div>
                    <div className="text-[13px] text-text-primary">{entry.video_name}</div>
                    <div className="text-[11px] text-text-muted">
                      {formatDuration(entry.duration_seconds || 0)} · {entry.segments.length}{" "}
                      segments
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <ExportMenu transcriptionId={entry.id} onExport={exportTranscription} />
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteTranscription(entry.id);
                    }}
                    className="btn btn-ghost btn-sm p-1"
                  >
                    <Trash2 size={14} className="text-text-muted hover:text-danger" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
