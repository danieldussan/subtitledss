interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  variant?: "text" | "rect" | "circle";
  className?: string;
}

const variantClasses = {
  text: "h-4 w-full rounded",
  rect: "rounded-lg",
  circle: "rounded-full",
};

export function Skeleton({ width, height, variant = "text", className = "" }: SkeletonProps) {
  return (
    <div
      className={`skeleton ${variantClasses[variant]} ${className}`}
      style={{ width, height }}
    />
  );
}
