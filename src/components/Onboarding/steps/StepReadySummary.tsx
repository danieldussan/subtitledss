import type { OnboardingSelections } from "../../../hooks/useOnboarding";

interface StepReadySummaryProps {
  selections: OnboardingSelections;
}

const modelLabels: Record<string, string> = {
  tiny: "Tiny (39 MB)",
  base: "Base (142 MB)",
  small: "Small (466 MB)",
  medium: "Medium (1.5 GB)",
};

const langLabels: Record<string, string> = {
  es: "Spanish",
  en: "English",
};

export function StepReadySummary({ selections }: StepReadySummaryProps) {
  return (
    <div className="space-y-4">
      <div className="bg-bg-base rounded-xl p-5 space-y-4">
        <div className="flex items-center gap-3">
          <div className="w-7 h-7 rounded-lg bg-success-subtle flex items-center justify-center shrink-0">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-success">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <div>
            <div className="text-[11px] text-text-muted">Model</div>
            <div className="text-[13px] font-medium text-text-primary">{modelLabels[selections.model] || selections.model}</div>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <div className="w-7 h-7 rounded-lg bg-success-subtle flex items-center justify-center shrink-0">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-success">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <div>
            <div className="text-[11px] text-text-muted">Language</div>
            <div className="text-[13px] font-medium text-text-primary">{langLabels[selections.language] || selections.language}</div>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <div className="w-7 h-7 rounded-lg bg-success-subtle flex items-center justify-center shrink-0">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-success">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <div>
            <div className="text-[11px] text-text-muted">Translation</div>
            <div className="text-[13px] font-medium text-text-primary">
              {selections.translationEnabled
                ? `${langLabels[selections.translationDirection.source] || selections.translationDirection.source} → ${langLabels[selections.translationDirection.target] || selections.translationDirection.target}`
                : "Disabled"}
            </div>
          </div>
        </div>
      </div>

      <div className="text-center text-[13px] text-text-secondary">
        Press <kbd className="px-1.5 py-0.5 bg-bg-surface border border-border-default rounded text-xs font-mono">Ctrl+Shift+S</kbd> to start capturing
      </div>
    </div>
  );
}
