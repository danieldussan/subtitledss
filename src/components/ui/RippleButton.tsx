import { useState, type ButtonHTMLAttributes, type ReactNode } from "react";

interface RippleButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  variant?: "primary" | "ghost" | "danger";
  size?: "sm" | "md";
}

interface Ripple {
  id: number;
  x: number;
  y: number;
}

const variantClasses = {
  primary: "btn-primary",
  ghost: "btn-ghost",
  danger: "btn-danger",
};

const sizeClasses = {
  sm: "btn-sm",
  md: "",
};

export function RippleButton({
  children,
  variant = "primary",
  size = "md",
  className = "",
  onClick,
  ...props
}: RippleButtonProps) {
  const [ripples, setRipples] = useState<Ripple[]>([]);

  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const id = Date.now();

    setRipples((prev) => [...prev, { id, x, y }]);
    setTimeout(() => {
      setRipples((prev) => prev.filter((r) => r.id !== id));
    }, 600);

    onClick?.(e);
  };

  return (
    <button
      className={`btn ${variantClasses[variant]} ${sizeClasses[size]} relative overflow-hidden ${className}`}
      onClick={handleClick}
      {...props}
    >
      {ripples.map((ripple) => (
        <span
          key={ripple.id}
          className="absolute rounded-full bg-white/20 pointer-events-none animate-[ripple_0.6s_ease-out_forwards]"
          style={{
            left: ripple.x - 10,
            top: ripple.y - 10,
            width: 20,
            height: 20,
          }}
        />
      ))}
      <span className="relative z-10">{children}</span>
    </button>
  );
}
