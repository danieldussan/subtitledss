import { useState } from "react";
import { MonitorPlay, MonitorOff, Circle, Languages, Brain } from "lucide-react";
import { Sidebar } from "./Sidebar";
import { SectionRouter } from "./SectionRouter";
import { ToastContainer } from "../ui/Toast";
import { useToast } from "../../hooks/useToast";
import type { Section } from "./types";

interface AppShellProps {
  isCapturing: boolean;
  overlayVisible: boolean;
  translationEnabled: boolean;
  loadedModel: string | null;
  audioDevice: string | null;
  onToggleCapture: () => void;
  onToggleOverlay: () => void;
  onToggleTranslation: () => void;
}

export function AppShell({
  isCapturing,
  overlayVisible,
  translationEnabled,
  loadedModel,
  audioDevice,
  onToggleCapture,
  onToggleOverlay,
  onToggleTranslation,
}: AppShellProps) {
  const [activeSection, setActiveSection] = useState<Section>("dashboard");
  const toast = useToast();

  return (
    <div className="h-screen flex bg-bg-base">
      <Sidebar activeSection={activeSection} onNavigate={setActiveSection} />

      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3 border-b border-border-subtle bg-bg-raised/80 backdrop-blur-sm">
          <div className="text-sm font-semibold text-text-primary capitalize">{activeSection}</div>

          <div className="flex items-center gap-2">
            <button
              onClick={onToggleOverlay}
              className="btn btn-ghost btn-sm gap-2"
              title={overlayVisible ? "Hide overlay" : "Show overlay"}
            >
              {overlayVisible ? <MonitorPlay size={15} /> : <MonitorOff size={15} />}
              <span className="hidden sm:inline">Overlay</span>
            </button>

            <button
              onClick={onToggleTranslation}
              className={`btn btn-ghost btn-sm gap-2 ${translationEnabled ? "text-accent" : ""}`}
              title={
                translationEnabled
                  ? "Disable translation (Ctrl+Shift+T)"
                  : "Enable translation (Ctrl+Shift+T)"
              }
            >
              <Languages size={15} />
              <span className="hidden sm:inline">Translate</span>
            </button>

            <div className="w-px h-5 bg-border-default" />

            <button
              onClick={onToggleCapture}
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

        {/* Main Content */}
        <main className="flex-1 overflow-y-auto" style={{ background: "var(--color-bg-base)" }}>
          <SectionRouter
            section={activeSection}
            isCapturing={isCapturing}
            overlayVisible={overlayVisible}
            translationEnabled={translationEnabled}
            loadedModel={loadedModel}
            audioDevice={audioDevice}
            onToggleOverlay={onToggleOverlay}
            onNavigate={setActiveSection}
          />
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
                <span>·</span>
                <span>{audioDevice}</span>
              </div>
            )}
          </div>
          <span>Ctrl+Shift+S</span>
        </footer>
      </div>

      <ToastContainer toasts={toast.toasts} onRemove={toast.removeToast} />
    </div>
  );
}
