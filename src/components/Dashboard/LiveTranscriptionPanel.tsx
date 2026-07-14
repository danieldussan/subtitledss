import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { MonitorPlay } from "lucide-react";

interface LiveTranscriptionPanelProps {
  isCapturing: boolean;
  overlayVisible: boolean;
}

interface TranscriptionPayload {
  id: number;
  text: string;
  translation: string | null;
  start: number;
  end: number;
  speed_ratio: number;
}

const MAX_LINES = 20;

export function LiveTranscriptionPanel({
  isCapturing,
  overlayVisible,
}: LiveTranscriptionPanelProps) {
  const [lines, setLines] = useState<TranscriptionPayload[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (overlayVisible) return;

    const unlisten = listen<TranscriptionPayload>("transcription", (event) => {
      const payload = event.payload;
      if (!payload.text) return;

      setLines((prev) => {
        const next = [...prev, payload];
        return next.length > MAX_LINES ? next.slice(-MAX_LINES) : next;
      });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [overlayVisible]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [lines]);

  useEffect(() => {
    if (isCapturing) {
      setLines([]);
    }
  }, [isCapturing]);

  return (
    <div className="card flex flex-col h-full min-h-[200px]">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border-subtle">
        <span className="text-[13px] font-semibold text-text-primary">Live Transcription</span>
        {overlayVisible ? (
          <span className="text-[11px] font-semibold px-2.5 py-1 rounded-full bg-accent-subtle text-accent">
            Overlay Active
          </span>
        ) : isCapturing ? (
          <span className="text-[11px] font-semibold px-2.5 py-1 rounded-full bg-success-subtle text-success flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-success animate-pulse" />
            Streaming
          </span>
        ) : null}
      </div>
      <div ref={scrollRef} className="flex-1 overflow-y-auto p-4">
        {overlayVisible ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="w-10 h-10 rounded-xl bg-accent-subtle inline-flex items-center justify-center mb-2.5">
                <MonitorPlay size={20} className="text-accent" />
              </div>
              <div className="text-[13px] font-medium text-text-primary mb-1">
                Overlay is active
              </div>
              <div className="text-[12px] text-text-muted">
                Subtitles are displayed on screen via the overlay window.
              </div>
              <div className="text-[11px] text-text-muted mt-2">
                Disable overlay to view transcriptions here.
              </div>
            </div>
          </div>
        ) : isCapturing && lines.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-text-muted">
              <div className="text-[12px]">Waiting for audio input...</div>
            </div>
          </div>
        ) : lines.length > 0 ? (
          <div className="space-y-2">
            {lines.map((line) => (
              <div key={line.id} className="text-[13px] text-text-primary leading-relaxed">
                {line.text}
                {line.translation && (
                  <div className="text-[12px] text-accent mt-0.5">{line.translation}</div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-text-muted">
              <div className="text-[12px]">Start capturing to see transcriptions</div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
