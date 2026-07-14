interface StepLanguageSelectionProps {
  selectedLanguage: string;
  onSelect: (language: string) => void;
}

const languages = [
  { value: "es", label: "Spanish", flag: "\ud83c\uddea\ud83c\uddf8" },
  { value: "en", label: "English", flag: "\ud83c\uddfa\ud83c\uddf8" },
  { value: "pt", label: "Portuguese", flag: "\ud83c\udde7\ud83c\uddf7" },
  { value: "fr", label: "French", flag: "\ud83c\uddeb\ud83c\uddf7" },
  { value: "de", label: "German", flag: "\ud83c\udde9\ud83c\uddea" },
  { value: "ja", label: "Japanese", flag: "\ud83c\uddef\ud83c\uddf5" },
];

export function StepLanguageSelection({ selectedLanguage, onSelect }: StepLanguageSelectionProps) {
  return (
    <div className="grid grid-cols-3 gap-3">
      {languages.map((lang) => (
        <button
          key={lang.value}
          onClick={() => onSelect(lang.value)}
          className={`p-4 rounded-xl text-center transition-all ${
            selectedLanguage === lang.value
              ? "bg-accent-subtle border-2 border-accent text-accent"
              : "bg-bg-base border border-border-subtle hover:border-border-default"
          }`}
        >
          <div className="text-2xl mb-1.5">{lang.flag}</div>
          <div className="text-[13px] font-medium">{lang.label}</div>
        </button>
      ))}
    </div>
  );
}
