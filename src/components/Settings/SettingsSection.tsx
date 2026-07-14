import { useState } from "react";
import { ChevronDown } from "lucide-react";

interface SettingsSectionProps {
  title: string;
  description?: string;
  defaultCollapsed?: boolean;
  children: React.ReactNode;
}

export function SettingsSection({
  title,
  description,
  defaultCollapsed = false,
  children,
}: SettingsSectionProps) {
  const [collapsed, setCollapsed] = useState(defaultCollapsed);

  return (
    <div className="border-b border-border-subtle last:border-b-0">
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="w-full flex items-center justify-between px-5 py-4 text-left hover:bg-bg-hover/30 transition-colors"
      >
        <div>
          <div className="text-[15px] font-semibold text-text-primary">{title}</div>
          {description && (
            <div className="text-xs text-text-muted mt-0.5">{description}</div>
          )}
        </div>
        <ChevronDown
          size={16}
          className={`text-text-muted transition-transform ${collapsed ? "" : "rotate-180"}`}
        />
      </button>
      {!collapsed && <div className="px-5 pb-4">{children}</div>}
    </div>
  );
}
