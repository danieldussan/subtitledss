import { User, Bot } from "lucide-react";

interface AiChatMessageProps {
  role: "user" | "assistant";
  content: string;
}

export function AiChatMessage({ role, content }: AiChatMessageProps) {
  const isUser = role === "user";

  return (
    <div className={`flex gap-3 ${isUser ? "justify-end" : "justify-start"}`}>
      {!isUser && (
        <div className="w-7 h-7 rounded-full bg-accent-subtle flex items-center justify-center flex-shrink-0">
          <Bot size={14} className="text-accent" />
        </div>
      )}

      <div
        className={`max-w-[80%] px-4 py-3 rounded-2xl text-[13px] leading-relaxed ${
          isUser
            ? "bg-accent text-white rounded-br-md"
            : "bg-bg-surface text-text-primary rounded-bl-md border border-border-subtle"
        }`}
      >
        <div className="whitespace-pre-wrap">{content}</div>
      </div>

      {isUser && (
        <div className="w-7 h-7 rounded-full bg-bg-surface flex items-center justify-center flex-shrink-0 border border-border-subtle">
          <User size={14} className="text-text-muted" />
        </div>
      )}
    </div>
  );
}
