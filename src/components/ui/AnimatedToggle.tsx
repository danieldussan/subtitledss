interface AnimatedToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
}

export function AnimatedToggle({ checked, onChange, label, disabled }: AnimatedToggleProps) {
  return (
    <button
      onClick={() => !disabled && onChange(!checked)}
      disabled={disabled}
      className={`inline-flex items-center gap-2.5 ${disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
      role="switch"
      aria-checked={checked}
    >
      <div
        className={`relative w-11 h-6 rounded-full transition-colors ${
          checked ? "bg-accent" : "bg-border-default"
        }`}
      >
        <div
          className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform ${
            checked ? "translate-x-[22px]" : ""
          }`}
          style={{ transition: "transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)" }}
        />
      </div>
      {label && <span className="text-[13px] font-medium">{label}</span>}
    </button>
  );
}
