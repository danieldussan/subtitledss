import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AppShell } from "./components/Layout/AppShell";
import { OnboardingWizard } from "./components/Onboarding/OnboardingWizard";
import { AppConfig } from "./hooks/useSettings";
import { useToast } from "./hooks/useToast";

function App() {
  const [isCapturing, setIsCapturing] = useState(false);
  const [overlayVisible, setOverlayVisible] = useState(true);
  const [translationEnabled, setTranslationEnabled] = useState(false);
  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [audioDevice, setAudioDevice] = useState<string | null>(null);
  const toast = useToast();
  const toggleCaptureRef = useRef<() => void>(() => {});
  const toggleOverlayRef = useRef<() => void>(() => {});
  const toggleTranslationRef = useRef<() => void>(() => {});

  const toggleCapture = useCallback(async () => {
    try {
      if (isCapturing) {
        await invoke("stop_capture");
        setIsCapturing(false);
        toast.success("Capture stopped");
      } else {
        await invoke("start_capture");
        setIsCapturing(true);
        toast.success("Capture started");
      }
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to toggle capture";
      toast.error(msg);
      console.error("Failed to toggle capture:", err);
    }
  }, [isCapturing, toast]);

  const toggleOverlay = useCallback(async () => {
    try {
      await invoke("toggle_overlay");
      setOverlayVisible((prev) => !prev);
    } catch (err) {
      toast.error("Failed to toggle overlay");
      console.error("Failed to toggle overlay:", err);
    }
  }, [toast]);

  const toggleTranslation = useCallback(async () => {
    try {
      const config = await invoke<AppConfig>("get_config");
      const newEnabled = !config.translation.enabled;
      await invoke("save_config", {
        config: { ...config, translation: { ...config.translation, enabled: newEnabled } },
      });
      setTranslationEnabled(newEnabled);
      toast.success(`Translation ${newEnabled ? "enabled" : "disabled"}`);
    } catch (err) {
      toast.error("Failed to toggle translation");
      console.error("Failed to toggle translation:", err);
    }
  }, [toast]);

  useEffect(() => {
    toggleCaptureRef.current = toggleCapture;
  }, [toggleCapture]);

  useEffect(() => {
    toggleOverlayRef.current = toggleOverlay;
  }, [toggleOverlay]);

  useEffect(() => {
    toggleTranslationRef.current = toggleTranslation;
  }, [toggleTranslation]);

  useEffect(() => {
    loadModelState();
    loadAudioDevice();

    const unlistenCapture = listen("toggle-capture", () => {
      toggleCaptureRef.current();
    });
    const unlistenOverlay = listen("toggle-overlay", () => {
      toggleOverlayRef.current();
    });
    const unlistenTranslation = listen("toggle-translation", () => {
      toggleTranslationRef.current();
    });

    return () => {
      unlistenCapture.then((fn) => fn());
      unlistenOverlay.then((fn) => fn());
      unlistenTranslation.then((fn) => fn());
    };
  }, []);

  const loadModelState = async () => {
    try {
      const config = await invoke<{ whisper: { model: string } }>("get_config");
      setLoadedModel(config.whisper.model);
    } catch (err) {
      console.error("Failed to load model state:", err);
    }
  };

  const loadAudioDevice = async () => {
    try {
      const config = await invoke<{ audio: { device: string | null } }>("get_config");
      setAudioDevice(config.audio.device);
    } catch (err) {
      console.error("Failed to load audio device:", err);
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === "S") {
        e.preventDefault();
        toggleCapture();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isCapturing]);

  return (
    <>
      <OnboardingWizard />
      <AppShell
        isCapturing={isCapturing}
        overlayVisible={overlayVisible}
        translationEnabled={translationEnabled}
        loadedModel={loadedModel}
        audioDevice={audioDevice}
        onToggleCapture={toggleCapture}
        onToggleOverlay={toggleOverlay}
        onToggleTranslation={toggleTranslation}
      />
    </>
  );
}

export default App;
