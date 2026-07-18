import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Brain, Send, Loader2, MessageSquare } from "lucide-react";
import { AiChatMessage } from "./AiChatMessage";

interface AiPanelProps {
  transcriptionId: number;
  transcriptionText: string;
  language?: string;
  targetLanguage?: string;
  summary: string | null;
  onSummaryUpdate?: (summary: string) => void;
}

type AiTab = "summary" | "chat";

interface ChatMsg {
  role: "user" | "assistant";
  content: string;
}

export function AiPanel({
  transcriptionId: _transcriptionId,
  transcriptionText,
  language,
  targetLanguage,
  summary: initialSummary,
  onSummaryUpdate,
}: AiPanelProps) {
  const [activeTab, setActiveTab] = useState<AiTab>("summary");
  const [summary, setSummary] = useState(initialSummary || "");
  const [summaryLoading, setSummaryLoading] = useState(false);
  const [chatMessages, setChatMessages] = useState<ChatMsg[]>([]);
  const [chatInput, setChatInput] = useState("");
  const [chatLoading, setChatLoading] = useState(false);
  const chatEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [chatMessages]);

  useEffect(() => {
    const unlisten = listen<{ token: string }>("ai-chat-token", (event) => {
      setChatMessages((prev) => {
        const last = prev[prev.length - 1];
        if (last && last.role === "assistant") {
          return [...prev.slice(0, -1), { ...last, content: last.content + event.payload.token }];
        }
        return [...prev, { role: "assistant", content: event.payload.token }];
      });
    });

    const unlistenDone = listen("ai-chat-done", () => {
      setChatLoading(false);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenDone.then((fn) => fn());
    };
  }, []);

  const handleSummarize = useCallback(async () => {
    try {
      setSummaryLoading(true);
      const result = await invoke<string>("ai_summarize", {
        transcriptionText,
        customPrompt: null,
        language: targetLanguage || language || null,
      });
      setSummary(result);
      onSummaryUpdate?.(result);
    } catch (err) {
      console.error("Summary failed:", err);
      setSummary("Failed to generate summary. Check your AI provider settings.");
    } finally {
      setSummaryLoading(false);
    }
  }, [transcriptionText, language, targetLanguage, onSummaryUpdate]);

  const handleSendChat = useCallback(async () => {
    if (!chatInput.trim() || chatLoading) return;

    const userMsg = chatInput.trim();
    setChatInput("");
    setChatMessages((prev) => [...prev, { role: "user", content: userMsg }]);
    setChatLoading(true);

    try {
      await invoke("ai_chat_stream_start", {
        message: userMsg,
        systemContext: transcriptionText,
        history: chatMessages.map((m) => ({
          role: m.role,
          content: m.content,
        })),
        language: targetLanguage || language || null,
      });
    } catch (err) {
      console.error("Chat failed:", err);
      setChatMessages((prev) => [
        ...prev,
        { role: "assistant", content: "Failed to get response. Check your AI provider settings." },
      ]);
      setChatLoading(false);
    }
  }, [chatInput, chatLoading, chatMessages, transcriptionText, language, targetLanguage]);

  return (
    <div className="card p-5">
      <div className="flex items-center gap-2 mb-4">
        <Brain size={18} className="text-accent" />
        <div className="section-title">AI Assistant</div>
      </div>

      <div className="tab-bar mb-4">
        <button
          onClick={() => setActiveTab("summary")}
          className={`tab-item ${activeTab === "summary" ? "active" : ""}`}
        >
          Summary
        </button>
        <button
          onClick={() => setActiveTab("chat")}
          className={`tab-item ${activeTab === "chat" ? "active" : ""}`}
        >
          <MessageSquare size={14} className="inline mr-1" />
          Chat
        </button>
      </div>

      {activeTab === "summary" && (
        <div>
          {summary ? (
            <div className="text-[13px] text-text-secondary leading-relaxed whitespace-pre-wrap">
              {summary}
            </div>
          ) : (
            <div className="text-center py-8">
              <button
                onClick={handleSummarize}
                disabled={summaryLoading}
                className="btn btn-primary gap-2"
              >
                {summaryLoading ? (
                  <Loader2 size={16} className="animate-spin" />
                ) : (
                  <Brain size={16} />
                )}
                {summaryLoading ? "Generating..." : "Generate Summary"}
              </button>
            </div>
          )}
        </div>
      )}

      {activeTab === "chat" && (
        <div className="flex flex-col h-[300px]">
          <div className="flex-1 overflow-y-auto space-y-3 pr-2">
            {chatMessages.length === 0 && (
              <div className="text-center py-8 text-[13px] text-text-muted">
                Ask questions about this transcription
              </div>
            )}
            {chatMessages.map((msg, i) => (
              <AiChatMessage key={i} role={msg.role} content={msg.content} />
            ))}
            {chatLoading && chatMessages[chatMessages.length - 1]?.role !== "assistant" && (
              <div className="flex gap-3">
                <div className="w-7 h-7 rounded-full bg-accent-subtle flex items-center justify-center">
                  <Loader2 size={14} className="text-accent animate-spin" />
                </div>
              </div>
            )}
            <div ref={chatEndRef} />
          </div>

          <div className="flex gap-2 mt-3 pt-3 border-t border-border-subtle">
            <input
              type="text"
              value={chatInput}
              onChange={(e) => setChatInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSendChat()}
              placeholder="Ask about the transcription..."
              className="input flex-1"
              disabled={chatLoading}
            />
            <button
              onClick={handleSendChat}
              disabled={!chatInput.trim() || chatLoading}
              className="btn btn-primary btn-sm"
            >
              <Send size={14} />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
