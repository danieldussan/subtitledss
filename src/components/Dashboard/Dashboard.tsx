import { Brain, Languages, Mic } from "lucide-react";
import { StatusHeroCard } from "./StatusHeroCard";
import { QuickActions } from "./QuickActions";
import { LiveTranscriptionPanel } from "./LiveTranscriptionPanel";
import { AudioMeterCard } from "./AudioMeterCard";
import { ModelStatusCard } from "./ModelStatusCard";
import type { Section } from "../Layout/types";

interface DashboardProps {
  isCapturing: boolean;
  overlayVisible: boolean;
  translationEnabled: boolean;
  loadedModel: string | null;
  audioDevice: string | null;
  onToggleOverlay: () => void;
  onNavigate: (section: Section) => void;
}

export function Dashboard({
  isCapturing,
  overlayVisible,
  translationEnabled,
  loadedModel,
  audioDevice,
  onToggleOverlay,
  onNavigate,
}: DashboardProps) {
  return (
    <div className="flex flex-col gap-6 p-6">
      <div className="grid grid-cols-3 gap-4">
        <StatusHeroCard
          icon={Brain}
          title="Whisper Model"
          value={loadedModel || "None"}
          detail={loadedModel ? "142 MB · 8 threads · CPU" : "No model loaded"}
          status={loadedModel ? "active" : "inactive"}
          onClick={() => onNavigate("settings")}
        />
        <StatusHeroCard
          icon={Languages}
          title="Translation"
          value={translationEnabled ? "ES → EN" : "Disabled"}
          detail={translationEnabled ? "Marian MT · Ready" : "Enable in settings"}
          status={translationEnabled ? "active" : "inactive"}
          onClick={() => onNavigate("settings")}
        />
        <StatusHeroCard
          icon={Mic}
          title="Audio"
          value={isCapturing ? "Active" : "Idle"}
          detail={audioDevice || "No device configured"}
          status={isCapturing ? "active" : "inactive"}
          onClick={() => onNavigate("audio")}
        />
      </div>

      <div>
        <div className="text-[12px] font-semibold text-text-muted uppercase tracking-wider mb-3">
          Quick Actions
        </div>
        <QuickActions
          overlayVisible={overlayVisible}
          onToggleOverlay={onToggleOverlay}
          onNavigate={onNavigate}
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <LiveTranscriptionPanel
          isCapturing={isCapturing}
          overlayVisible={overlayVisible}
        />
        <div className="flex flex-col gap-4">
          <AudioMeterCard isCapturing={isCapturing} />
          <ModelStatusCard loadedModel={loadedModel} onNavigate={onNavigate} />
        </div>
      </div>
    </div>
  );
}
