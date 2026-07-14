import type { LucideIcon } from "lucide-react";

interface StatusHeroCardProps {
  icon: LucideIcon;
  title: string;
  value: string;
  detail: string;
  status: "active" | "inactive" | "warning";
  onClick?: () => void;
}

const statusConfig = {
  active: { dot: "status-dot active", text: "text-success" },
  inactive: { dot: "status-dot inactive", text: "text-text-muted" },
  warning: { dot: "status-dot", text: "text-warning" },
} as const;

export function StatusHeroCard({
  icon: Icon,
  title,
  value,
  detail,
  status,
  onClick,
}: StatusHeroCardProps) {
  const cfg = statusConfig[status];
  return (
    <button
      onClick={onClick}
      className="card flex flex-col p-5 text-left w-full cursor-pointer transition-all hover:border-border-default"
    >
      <div className="flex items-center justify-between mb-2">
        <span className="text-[12px] font-medium text-text-muted uppercase tracking-wide">
          {title}
        </span>
        <div
          className={`w-8 h-8 rounded-lg flex items-center justify-center ${cfg.text}`}
          style={{ background: "var(--color-accent-subtle)" }}
        >
          <Icon size={16} />
        </div>
      </div>
      <div className="text-[22px] font-bold leading-tight text-text-primary">{value}</div>
      <div className="text-[12px] text-text-muted mt-1">{detail}</div>
    </button>
  );
}
