import {
  LayoutDashboard,
  MessageSquareText,
  Download,
  Mic,
  Settings2,
  MonitorPlay,
  Keyboard,
  type LucideIcon,
} from "lucide-react";

export type Section =
  | "dashboard"
  | "transcriptions"
  | "export"
  | "audio"
  | "settings"
  | "overlay"
  | "shortcuts";

export interface NavItem {
  id: Section;
  label: string;
  icon: LucideIcon;
  group: "main" | "configure";
}

export const SECTION_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, group: "main" },
  { id: "transcriptions", label: "Transcriptions", icon: MessageSquareText, group: "main" },
  { id: "export", label: "Export", icon: Download, group: "main" },
  { id: "audio", label: "Audio", icon: Mic, group: "configure" },
  { id: "overlay", label: "Overlay", icon: MonitorPlay, group: "configure" },
  { id: "shortcuts", label: "Shortcuts", icon: Keyboard, group: "configure" },
  { id: "settings", label: "Settings", icon: Settings2, group: "configure" },
];
