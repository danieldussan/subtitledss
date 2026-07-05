import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SettingsPanel } from "./components/Settings/SettingsPanel";
import { HistoryList } from "./components/History/HistoryList";
import { ModelList } from "./components/ModelManager/ModelList";
import { ToastContainer } from "./components/ui/Toast";
import { useToast } from "./hooks/useToast";
import { Settings2, History, Brain, MonitorPlay, MonitorOff, Circle } from "lucide-react";

type Tab = "settings" | "history" | "models";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("settings");
  const [isCapturing, setIsCapturing] = useState(false);
  const [overlayVisible, setOverlayVisible] = useState(true);
  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [audioDevice, setAudioDevice] = useState<string | null>(null);
  const toast = useToast();
  const toggleCaptureRef = useRef<() => void>(() => {});
  const toggleOverlayRef = useRef<() => void>(() => {});

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

  useEffect(() => {
    toggleCaptureRef.current = toggleCapture;
  }, [toggleCapture]);

  useEffect(() => {
    toggleOverlayRef.current = toggleOverlay;
  }, [toggleOverlay]);

  useEffect(() => {
    loadModelState();
    loadAudioDevice();

    const unlistenCapture = listen("toggle-capture", () => {
      toggleCaptureRef.current();
    });
    const unlistenOverlay = listen("toggle-overlay", () => {
      toggleOverlayRef.current();
    });

    return () => {
      unlistenCapture.then((fn) => fn());
      unlistenOverlay.then((fn) => fn());
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
    <div className="h-screen flex flex-col bg-bg-base">
      {/* Header */}
      <header className="flex items-center justify-between px-5 py-3 border-b border-border-subtle bg-bg-raised/80 backdrop-blur-sm">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-accent/20 flex items-center justify-center">
            <span className="text-accent font-bold text-sm">S</span>
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-semibold text-text-primary leading-tight">
              subtitledss
            </span>
            <span className="text-[11px] text-text-muted leading-tight">real-time subtitles</span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={toggleOverlay}
            className="btn btn-ghost btn-sm gap-2"
            title={overlayVisible ? "Hide overlay" : "Show overlay"}
          >
            {overlayVisible ? <MonitorPlay size={15} /> : <MonitorOff size={15} />}
            <span className="hidden sm:inline">Overlay</span>
          </button>

          <div className="w-px h-5 bg-border-default" />

          <button
            onClick={toggleCapture}
            className={`btn btn-sm gap-2 ${
              isCapturing ? "bg-danger-subtle text-danger" : "btn-primary"
            }`}
            title={isCapturing ? "Stop capture (Ctrl+Shift+S)" : "Start capture (Ctrl+Shift+S)"}
          >
            <Circle size={8} className={isCapturing ? "fill-danger animate-pulse" : ""} />
            <span>{isCapturing ? "Stop" : "Start"}</span>
          </button>
        </div>
      </header>

      {/* Tab Navigation */}
      <nav className="px-5 pt-3 pb-0">
        <div className="tab-bar">
          {[
            { id: "settings" as const, label: "Settings", icon: Settings2 },
            { id: "history" as const, label: "History", icon: History },
            { id: "models" as const, label: "Models", icon: Brain },
          ].map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveTab(id)}
              className={`tab-item ${activeTab === id ? "active" : ""}`}
            >
              <Icon size={15} />
              {label}
            </button>
          ))}
        </div>
      </nav>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto">
        {activeTab === "settings" && <SettingsPanel isCapturing={isCapturing} />}
        {activeTab === "history" && <HistoryList />}
        {activeTab === "models" && <ModelList />}
      </main>

      {/* Status Bar */}
      <footer className="flex items-center justify-between px-5 py-2 border-t border-border-subtle bg-bg-raised/60 text-[11px] text-text-muted">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1.5">
            <div className={`status-dot ${isCapturing ? "active" : "inactive"}`} />
            <span>{isCapturing ? "Capturing" : "Idle"}</span>
          </div>
          <div className="flex items-center gap-1.5">
            <Brain size={12} className="text-text-muted" />
            <span>{loadedModel ? `Whisper: ${loadedModel}` : "Whisper: not loaded"}</span>
          </div>
          {audioDevice && (
            <div className="flex items-center gap-1.5">
              <span>•</span>
              <span>{audioDevice}</span>
            </div>
          )}
        </div>
        <span>Ctrl+Shift+S</span>
      </footer>

      {/* Toast Notifications */}
      <ToastContainer toasts={toast.toasts} onRemove={toast.removeToast} />
    </div>
  );
}

export default App;
