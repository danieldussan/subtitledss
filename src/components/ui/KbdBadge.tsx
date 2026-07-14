interface KbdBadgeProps {
  keys: string[];
}

export function KbdBadge({ keys }: KbdBadgeProps) {
  return (
    <span className="inline-flex items-center gap-1">
      {keys.map((key, i) => (
        <span key={i}>
          <kbd className="px-1.5 py-0.5 bg-bg-surface border border-border-default rounded text-xs font-mono text-text-secondary shadow-[0_1px_0_var(--color-border-default)]">
            {key}
          </kbd>
          {i < keys.length - 1 && <span className="text-text-muted mx-0.5">+</span>}
        </span>
      ))}
    </span>
  );
}
