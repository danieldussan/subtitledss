import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface TranscriptionResult {
  segments: TranscriptionSegment[];
  full_text: string;
}

export interface TranscriptionSegment {
  start: number;
  end: number;
  text: string;
}

export function useTranscription() {
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [result, setResult] = useState<TranscriptionResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const transcribe = useCallback(
    async (audioData: number[], language?: string, threads?: number) => {
      try {
        setIsTranscribing(true);
        setError(null);
        const res = await invoke<TranscriptionResult>("transcribe_audio", {
          audioData,
          language,
          threads,
        });
        setResult(res);
        return res;
      } catch (err) {
        setError(err as string);
        return null;
      } finally {
        setIsTranscribing(false);
      }
    },
    [],
  );

  const clearResult = useCallback(() => {
    setResult(null);
  }, []);

  return { isTranscribing, result, error, transcribe, clearResult };
}
