interface ModelOption {
  value: string;
  label: string;
  size: string;
  tier: "fast" | "balanced" | "precise";
  recommended?: boolean;
}

interface ModelPickerProps {
  selectedModel: string;
  onSelect: (model: string) => void;
}

const tierConfig = {
  fast: { label: "Fast", className: "bg-success-subtle text-success" },
  balanced: { label: "Balanced", className: "bg-accent-subtle text-accent" },
  precise: { label: "Precise", className: "bg-warning-subtle text-warning" },
};

const models: ModelOption[] = [
  { value: "tiny", label: "Tiny", size: "39 MB", tier: "fast" },
  { value: "base", label: "Base", size: "142 MB", tier: "balanced", recommended: true },
  { value: "small", label: "Small", size: "466 MB", tier: "balanced" },
  { value: "medium", label: "Medium", size: "1.5 GB", tier: "precise" },
  { value: "large-v3", label: "Large v3", size: "3.1 GB", tier: "precise" },
];

export function ModelPicker({ selectedModel, onSelect }: ModelPickerProps) {
  return (
    <div className="grid grid-cols-3 gap-2.5">
      {models.map((model) => (
        <button
          key={model.value}
          onClick={() => onSelect(model.value)}
          className={`relative p-3.5 rounded-xl text-left transition-all ${
            selectedModel === model.value
              ? "bg-accent-subtle border-2 border-accent"
              : "bg-bg-base border border-border-subtle hover:border-border-default"
          }`}
        >
          {selectedModel === model.value && (
            <div className="absolute top-2.5 right-2.5 w-4 h-4 rounded-full bg-accent flex items-center justify-center">
              <svg
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="none"
                stroke="white"
                strokeWidth="3"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </div>
          )}
          <div className="text-[13px] font-semibold mb-0.5 text-text-primary">{model.label}</div>
          <div className="text-[11px] text-text-muted mb-2">{model.size}</div>
          <span
            className={`text-[10px] font-semibold px-2 py-0.5 rounded-md ${tierConfig[model.tier].className}`}
          >
            {tierConfig[model.tier].label}
          </span>
          {model.recommended && (
            <div className="text-[10px] font-semibold text-success mt-1.5">Recommended</div>
          )}
        </button>
      ))}
    </div>
  );
}
