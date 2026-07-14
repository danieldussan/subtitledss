interface StepModelSelectionProps {
  selectedModel: string;
  onSelect: (model: string) => void;
}

const models = [
  { value: "tiny", label: "Tiny", size: "39 MB", tier: "fast" as const },
  { value: "base", label: "Base", size: "142 MB", tier: "balanced" as const, recommended: true },
  { value: "small", label: "Small", size: "466 MB", tier: "balanced" as const },
  { value: "medium", label: "Medium", size: "1.5 GB", tier: "precise" as const },
];

const tierConfig = {
  fast: { label: "Fast", className: "bg-success-subtle text-success" },
  balanced: { label: "Balanced", className: "bg-accent-subtle text-accent" },
  precise: { label: "Precise", className: "bg-warning-subtle text-warning" },
};

export function StepModelSelection({ selectedModel, onSelect }: StepModelSelectionProps) {
  return (
    <div className="grid grid-cols-2 gap-3">
      {models.map((model) => (
        <button
          key={model.value}
          onClick={() => onSelect(model.value)}
          className={`relative p-5 rounded-xl text-left transition-all ${
            selectedModel === model.value
              ? "bg-accent-subtle border-2 border-accent"
              : "bg-bg-base border border-border-subtle hover:border-border-default"
          }`}
        >
          {selectedModel === model.value && (
            <div className="absolute top-3 right-3 w-5 h-5 rounded-full bg-accent flex items-center justify-center">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="3">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </div>
          )}
          <div className="text-[14px] font-semibold mb-1 text-text-primary">{model.label}</div>
          <div className="text-xs text-text-muted mb-2.5">{model.size}</div>
          <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-md ${tierConfig[model.tier].className}`}>
            {tierConfig[model.tier].label}
          </span>
          {model.recommended && (
            <div className="text-[11px] font-semibold text-success mt-2">Recommended</div>
          )}
        </button>
      ))}
    </div>
  );
}
