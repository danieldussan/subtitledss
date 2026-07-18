import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "./useSettings";

export interface OnboardingSelections {
  model: string;
  language: string;
  translationEnabled: boolean;
  translationDirection: { source: string; target: string };
}

const defaultSelections: OnboardingSelections = {
  model: "base",
  language: "es",
  translationEnabled: true,
  translationDirection: { source: "es", target: "en" },
};

export function useOnboarding() {
  const [shouldShow, setShouldShow] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [selections, setSelections] = useState<OnboardingSelections>(defaultSelections);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkOnboardingStatus();
  }, []);

  const checkOnboardingStatus = async () => {
    try {
      const config = await invoke<AppConfig>("get_config");
      setShouldShow(!config.onboarding_completed);
    } catch (err) {
      console.error("Failed to check onboarding status:", err);
      setShouldShow(true);
    } finally {
      setLoading(false);
    }
  };

  const updateSelection = (partial: Partial<OnboardingSelections>) => {
    setSelections((prev) => ({ ...prev, ...partial }));
  };

  const complete = async () => {
    try {
      const config = await invoke<AppConfig>("get_config");
      await invoke("save_config", {
        config: {
          ...config,
          whisper: { ...config.whisper, model: selections.model, language: selections.language },
          translation: {
            ...config.translation,
            enabled: selections.translationEnabled,
            source_lang: selections.translationDirection.source,
            target_lang: selections.translationDirection.target,
          },
          onboarding_completed: true,
        },
      });
      setShouldShow(false);
    } catch (err) {
      console.error("Failed to complete onboarding:", err);
    }
  };

  const skip = async () => {
    try {
      const config = await invoke<AppConfig>("get_config");
      await invoke("save_config", {
        config: { ...config, onboarding_completed: true },
      });
      setShouldShow(false);
    } catch (err) {
      console.error("Failed to skip onboarding:", err);
    }
  };

  return {
    shouldShow,
    loading,
    currentStep,
    selections,
    setStep: setCurrentStep,
    updateSelection,
    complete,
    skip,
  };
}
