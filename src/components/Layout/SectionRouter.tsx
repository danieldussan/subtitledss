import { HistoryList } from "../History/HistoryList";
import { ModelList } from "../ModelManager/ModelList";
import { SettingsPanel } from "../Settings/SettingsPanel";
import { Dashboard } from "../Dashboard/Dashboard";
import { AudioPage } from "../Audio/AudioPage";
import { OverlayPage } from "../Overlay/OverlayPage";
import { ShortcutsPage } from "../Shortcuts/ShortcutsPage";
import { VideoTranscriptionPage } from "../VideoTranscription/VideoTranscriptionPage";
import type { Section } from "./types";

interface SectionRouterProps {
  section: Section;
  isCapturing: boolean;
  overlayVisible: boolean;
  translationEnabled: boolean;
  loadedModel: string | null;
  audioDevice: string | null;
  onToggleOverlay: () => void;
  onNavigate: (section: Section) => void;
}

export function SectionRouter({
  section,
  isCapturing,
  overlayVisible,
  translationEnabled,
  loadedModel,
  audioDevice,
  onToggleOverlay,
  onNavigate,
}: SectionRouterProps) {
  switch (section) {
    case "dashboard":
      return (
        <Dashboard
          isCapturing={isCapturing}
          overlayVisible={overlayVisible}
          translationEnabled={translationEnabled}
          loadedModel={loadedModel}
          audioDevice={audioDevice}
          onToggleOverlay={onToggleOverlay}
          onNavigate={onNavigate}
        />
      );
    case "transcriptions":
      return <HistoryList />;
    case "video":
      return <VideoTranscriptionPage />;
    case "export":
      return (
        <div className="flex items-center justify-center h-full text-text-muted">
          <div className="text-center">
            <div className="text-sm font-medium mb-1">Export</div>
            <div className="text-xs">Coming in Phase 1</div>
          </div>
        </div>
      );
    case "audio":
      return <AudioPage />;
    case "settings":
      return <SettingsPanel isCapturing={isCapturing} />;
    case "overlay":
      return <OverlayPage />;
    case "shortcuts":
      return <ShortcutsPage />;
    default:
      return <ModelList />;
  }
}
