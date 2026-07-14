interface ProgressDotsProps {
  currentStep: number;
  totalSteps: number;
}

export function ProgressDots({ currentStep, totalSteps }: ProgressDotsProps) {
  return (
    <div className="flex items-center justify-center gap-2">
      {Array.from({ length: totalSteps }).map((_, i) => (
        <div
          key={i}
          className={`h-2 rounded-full transition-all ${
            i < currentStep
              ? "w-2 bg-accent"
              : i === currentStep
                ? "w-6 bg-accent"
                : "w-2 bg-border-default"
          }`}
        />
      ))}
    </div>
  );
}
