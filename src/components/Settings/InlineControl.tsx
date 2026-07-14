interface InlineControlProps {
  label: string;
  description?: string;
  children: React.ReactNode;
}

export function InlineControl({ label, description, children }: InlineControlProps) {
  return (
    <div className="flex items-center justify-between py-3.5 border-b border-border-subtle last:border-b-0">
      <div className="flex-1 min-w-0 mr-4">
        <div className="text-[13px] font-medium text-text-primary">{label}</div>
        {description && (
          <div className="text-xs text-text-muted mt-0.5">{description}</div>
        )}
      </div>
      <div className="flex items-center gap-2 shrink-0">{children}</div>
    </div>
  );
}
