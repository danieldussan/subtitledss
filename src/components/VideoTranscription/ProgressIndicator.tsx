import { Loader2, Check } from "lucide-react";
import type { TranscriptionStep } from "../../hooks/useVideoTranscription";

interface ProgressIndicatorProps {
  step: TranscriptionStep;
  progress?: { progress: number; message: string } | null;
}

const STEPS: { key: TranscriptionStep; label: string }[] = [
  { key: "extracting", label: "Extracting Audio" },
  { key: "diarizing", label: "Detecting Speakers" },
  { key: "transcribing", label: "Transcribing" },
  { key: "saving", label: "Saving" },
];

export function ProgressIndicator({ step, progress }: ProgressIndicatorProps) {
  if (step === "idle" || step === "done" || step === "error") return null;

  const currentIndex = STEPS.findIndex((s) => s.key === step);

  return (
    <div className="card p-5">
      <div className="flex items-center gap-3 mb-4">
        <Loader2 size={18} className="text-accent animate-spin" />
        <span className="text-[13px] text-text-primary font-medium">
          {progress?.message || "Processing..."}
        </span>
      </div>

      <div className="flex items-center gap-2">
        {STEPS.map((s, i) => {
          const isComplete = i < currentIndex;
          const isCurrent = i === currentIndex;

          return (
            <div key={s.key} className="flex items-center gap-2">
              <div
                className={`w-6 h-6 rounded-full flex items-center justify-center text-[11px] font-medium ${
                  isComplete
                    ? "bg-success-subtle text-success"
                    : isCurrent
                      ? "bg-accent-subtle text-accent"
                      : "bg-bg-surface text-text-muted"
                }`}
              >
                {isComplete ? <Check size={12} /> : i + 1}
              </div>
              <span
                className={`text-[11px] ${isCurrent ? "text-text-primary" : "text-text-muted"}`}
              >
                {s.label}
              </span>
              {i < STEPS.length - 1 && <div className="w-8 h-px bg-border-default" />}
            </div>
          );
        })}
      </div>
    </div>
  );
}
