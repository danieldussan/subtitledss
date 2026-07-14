import { useOnboarding } from "../../hooks/useOnboarding";
import { ProgressDots } from "./ProgressDots";
import { StepModelSelection } from "./steps/StepModelSelection";
import { StepLanguageSelection } from "./steps/StepLanguageSelection";
import { StepTranslationSetup } from "./steps/StepTranslationSetup";
import { StepReadySummary } from "./steps/StepReadySummary";

const TOTAL_STEPS = 4;

export function OnboardingWizard() {
  const {
    shouldShow,
    loading,
    currentStep,
    selections,
    setStep,
    updateSelection,
    complete,
    skip,
  } = useOnboarding();

  if (loading || !shouldShow) return null;

  const canNext = () => {
    if (currentStep === 0) return !!selections.model;
    if (currentStep === 1) return !!selections.language;
    return true;
  };

  const handleNext = () => {
    if (currentStep < TOTAL_STEPS - 1) {
      setStep(currentStep + 1);
    } else {
      complete();
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setStep(currentStep - 1);
    }
  };

  const stepTitles = [
    "Choose your AI model",
    "What language will you hear?",
    "Enable translation?",
    "You're all set!",
  ];
  const stepDescs = [
    "Larger models are more accurate but use more CPU. You can change this later.",
    "Select the primary language for transcription.",
    "Translate transcriptions in real-time. Runs 100% locally on your machine.",
    "Here's your configuration summary.",
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-[540px] max-h-[90vh] bg-bg-raised border border-border-subtle rounded-2xl shadow-2xl flex flex-col overflow-hidden">
        <div className="flex flex-col items-center px-8 pt-8 pb-6">
          <div className="w-14 h-14 rounded-2xl bg-accent-subtle text-accent flex items-center justify-center font-bold text-2xl mb-5">
            S
          </div>
          <h1 className="text-xl font-bold text-text-primary mb-1.5">Welcome to subtitledss</h1>
          <p className="text-[13px] text-text-secondary text-center">
            Real-time subtitles powered by AI. Let's get you set up in under a minute.
          </p>
        </div>

        <div className="px-8 pb-6">
          <ProgressDots currentStep={currentStep} totalSteps={TOTAL_STEPS} />
        </div>

        <div className="flex-1 overflow-y-auto px-8 pb-6">
          <div className="mb-5">
            <div className="text-[11px] font-bold text-accent uppercase tracking-widest mb-1.5">
              Step {currentStep + 1} of {TOTAL_STEPS}
            </div>
            <h2 className="text-[17px] font-semibold text-text-primary mb-1">{stepTitles[currentStep]}</h2>
            <p className="text-[13px] text-text-secondary">{stepDescs[currentStep]}</p>
          </div>

          <div>
            {currentStep === 0 && (
              <StepModelSelection
                selectedModel={selections.model}
                onSelect={(model) => updateSelection({ model })}
              />
            )}
            {currentStep === 1 && (
              <StepLanguageSelection
                selectedLanguage={selections.language}
                onSelect={(language) => updateSelection({ language })}
              />
            )}
            {currentStep === 2 && (
              <StepTranslationSetup
                enabled={selections.translationEnabled}
                onToggle={(translationEnabled) => updateSelection({ translationEnabled })}
                sourceLang={selections.translationDirection.source}
                targetLang={selections.translationDirection.target}
                onDirectionChange={(source, target) =>
                  updateSelection({ translationDirection: { source, target } })
                }
              />
            )}
            {currentStep === 3 && <StepReadySummary selections={selections} />}
          </div>
        </div>

        <div className="flex items-center justify-between px-8 py-4 border-t border-border-subtle">
          <button
            onClick={skip}
            className="text-[12px] text-text-muted"
          >
            Skip setup
          </button>
          <div className="flex gap-2.5">
            {currentStep > 0 && (
              <button onClick={handleBack} className="btn btn-ghost btn-sm">
                Back
              </button>
            )}
            <button
              onClick={handleNext}
              disabled={!canNext()}
              className="btn btn-primary btn-sm"
            >
              {currentStep === TOTAL_STEPS - 1 ? "Get Started" : "Continue →"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
