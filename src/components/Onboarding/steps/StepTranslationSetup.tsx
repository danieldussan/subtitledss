interface StepTranslationSetupProps {
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  sourceLang: string;
  targetLang: string;
  onDirectionChange: (source: string, target: string) => void;
}

export function StepTranslationSetup({
  enabled,
  onToggle,
  sourceLang,
  targetLang,
  onDirectionChange,
}: StepTranslationSetupProps) {
  const languages = [
    { value: "es", label: "Spanish" },
    { value: "en", label: "English" },
  ];

  return (
    <div className="space-y-4">
      <button
        onClick={() => onToggle(!enabled)}
        className={`w-full flex items-center justify-between p-5 rounded-xl transition-all ${
          enabled
            ? "bg-accent-subtle border-2 border-accent"
            : "bg-bg-base border border-border-subtle"
        }`}
      >
        <div className="text-left">
          <div className="text-[14px] font-semibold text-text-primary">Real-time Translation</div>
          <div className="text-xs text-text-muted mt-0.5">Translate speech as it's captured</div>
        </div>
        <div
          className={`w-11 h-6 rounded-full transition-colors ${
            enabled ? "bg-accent" : "bg-border-default"
          }`}
        >
          <div
            className={`w-5 h-5 rounded-full bg-white shadow-sm transition-transform mt-0.5 ${
              enabled ? "translate-x-[22px]" : "translate-x-0.5"
            }`}
          />
        </div>
      </button>

      {enabled && (
        <div className="flex items-center justify-center gap-3 p-5 bg-bg-base rounded-xl border border-border-subtle">
          <select
            value={sourceLang}
            onChange={(e) => onDirectionChange(e.target.value, targetLang)}
            className="select w-auto min-w-[100px]"
          >
            {languages.map((l) => (
              <option key={l.value} value={l.value}>
                {l.label}
              </option>
            ))}
          </select>
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            width="16"
            height="16"
            className="text-text-muted shrink-0"
          >
            <path d="M5 12h14" />
            <path d="m12 5 7 7-7 7" />
          </svg>
          <select
            value={targetLang}
            onChange={(e) => onDirectionChange(sourceLang, e.target.value)}
            className="select w-auto min-w-[100px]"
          >
            {languages.map((l) => (
              <option key={l.value} value={l.value}>
                {l.label}
              </option>
            ))}
          </select>
        </div>
      )}

      <div className="text-center text-xs text-text-muted">Runs 100% locally on your machine</div>
    </div>
  );
}
