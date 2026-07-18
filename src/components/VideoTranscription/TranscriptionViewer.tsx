import type { DiarizedSegment } from "../../hooks/useVideoTranscription";

interface TranscriptionViewerProps {
  segments: DiarizedSegment[];
  fullText?: string;
}

const SPEAKER_COLORS = [
  "text-blue-400",
  "text-emerald-400",
  "text-amber-400",
  "text-purple-400",
  "text-rose-400",
  "text-cyan-400",
];

const SPEAKER_BG_COLORS = [
  "bg-blue-400/10",
  "bg-emerald-400/10",
  "bg-amber-400/10",
  "bg-purple-400/10",
  "bg-rose-400/10",
  "bg-cyan-400/10",
];

function formatTimestamp(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

function getSpeakerColor(speaker: string | null): number {
  if (!speaker) return 0;
  const match = speaker.match(/(\d+)/);
  return match ? parseInt(match[1]) % SPEAKER_COLORS.length : 0;
}

export function TranscriptionViewer({ segments, fullText: _fullText }: TranscriptionViewerProps) {
  if (!segments || segments.length === 0) {
    return (
      <div className="card p-5">
        <div className="section-title mb-2">Transcription</div>
        <p className="text-[13px] text-text-muted">
          No segments yet. Transcribe a video to see results.
        </p>
      </div>
    );
  }

  const hasSpeakers = segments.some((s) => s.speaker !== null);

  return (
    <div className="card p-5">
      <div className="flex items-center justify-between mb-4">
        <div className="section-title">Transcription</div>
        <div className="text-[11px] text-text-muted">{segments.length} segments</div>
      </div>

      <div className="space-y-2 max-h-[400px] overflow-y-auto pr-2">
        {segments.map((seg, i) => {
          const colorIdx = getSpeakerColor(seg.speaker);
          return (
            <div
              key={i}
              className={`flex gap-3 p-3 rounded-lg ${
                hasSpeakers ? SPEAKER_BG_COLORS[colorIdx] : "bg-bg-surface"
              }`}
            >
              <div className="flex-shrink-0 w-16 text-right">
                <span className="text-[11px] font-mono text-text-muted">
                  {formatTimestamp(seg.start)}
                </span>
              </div>
              <div className="flex-1 min-w-0">
                {hasSpeakers && seg.speaker && (
                  <div className={`text-[11px] font-medium mb-1 ${SPEAKER_COLORS[colorIdx]}`}>
                    {seg.speaker}
                  </div>
                )}
                <div className="text-[13px] text-text-primary leading-relaxed">{seg.text}</div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
