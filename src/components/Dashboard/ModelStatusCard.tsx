import { Brain } from "lucide-react";
import type { Section } from "../Layout/types";

interface ModelStatusCardProps {
  loadedModel: string | null;
  onNavigate: (section: Section) => void;
}

export function ModelStatusCard({ loadedModel, onNavigate }: ModelStatusCardProps) {
  return (
    <div className="card">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border-subtle">
        <span className="text-[13px] font-semibold text-text-primary">Models</span>
      </div>
      <div className="p-4">
        {loadedModel ? (
          <div className="flex items-center gap-3 p-3 bg-bg-base rounded-lg">
            <div className="w-8 h-8 rounded-lg bg-bg-surface flex items-center justify-center">
              <Brain size={16} className="text-text-muted" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-[13px] font-medium text-text-primary truncate">{loadedModel}</div>
              <div className="text-[11px] text-text-muted">Whisper</div>
            </div>
            <span className="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-success-subtle text-success">
              Loaded
            </span>
          </div>
        ) : (
          <button
            onClick={() => onNavigate("settings")}
            className="text-center w-full py-2 text-[12px] text-accent"
          >
            No model loaded — click to configure
          </button>
        )}
      </div>
    </div>
  );
}
