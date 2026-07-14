import { useRef, useState, type ReactNode } from "react";

interface GlowCardProps {
  children: ReactNode;
  className?: string;
}

export function GlowCard({ children, className = "" }: GlowCardProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ x: 0, y: 0 });
  const [visible, setVisible] = useState(false);

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    setPos({ x: e.clientX - rect.left, y: e.clientY - rect.top });
  };

  return (
    <div
      ref={ref}
      onMouseMove={handleMouseMove}
      onMouseEnter={() => setVisible(true)}
      onMouseLeave={() => setVisible(false)}
      className={`relative overflow-hidden rounded-xl border border-border-subtle bg-bg-raised transition-all hover:border-accent hover:shadow-[0_8px_32px_rgba(91,141,239,0.15)] ${className}`}
    >
      {visible && (
        <div
          className="pointer-events-none absolute inset-0 opacity-40 transition-opacity"
          style={{
            background: `radial-gradient(circle at ${pos.x}px ${pos.y}px, rgba(91,141,239,0.15) 0%, transparent 60%)`,
          }}
        />
      )}
      <div className="relative z-10">{children}</div>
    </div>
  );
}
