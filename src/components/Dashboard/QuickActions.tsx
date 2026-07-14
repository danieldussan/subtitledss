import { MonitorPlay, MonitorOff, MessageSquareText, Download, Settings2 } from "lucide-react";
import type { Section } from "../Layout/types";

interface QuickActionsProps {
  overlayVisible: boolean;
  onToggleOverlay: () => void;
  onNavigate: (section: Section) => void;
}

interface QuickAction {
  icon: typeof MonitorPlay;
  label: string;
  desc: string;
  onClick: () => void;
}

export function QuickActions({ overlayVisible, onToggleOverlay, onNavigate }: QuickActionsProps) {
  const actions: QuickAction[] = [
    {
      icon: overlayVisible ? MonitorPlay : MonitorOff,
      label: "Toggle Overlay",
      desc: overlayVisible ? "Hide overlay" : "Show overlay",
      onClick: onToggleOverlay,
    },
    {
      icon: MessageSquareText,
      label: "View History",
      desc: "Browse transcriptions",
      onClick: () => onNavigate("transcriptions"),
    },
    {
      icon: Download,
      label: "Export",
      desc: "SRT, VTT, TXT, JSON",
      onClick: () => onNavigate("export"),
    },
    {
      icon: Settings2,
      label: "Settings",
      desc: "Configure app",
      onClick: () => onNavigate("settings"),
    },
  ];

  return (
    <div className="grid grid-cols-4 gap-3">
      {actions.map((action) => (
        <button
          key={action.label}
          onClick={action.onClick}
          className="card flex flex-col gap-3 p-4 text-left cursor-pointer transition-all hover:border-border-default hover:-translate-y-0.5"
        >
          <div className="w-9 h-9 rounded-lg bg-bg-surface flex items-center justify-center">
            <action.icon size={18} className="text-accent" />
          </div>
          <div>
            <div className="text-[13px] font-semibold text-text-primary">{action.label}</div>
            <div className="text-[11px] text-text-muted mt-0.5">{action.desc}</div>
          </div>
        </button>
      ))}
    </div>
  );
}
