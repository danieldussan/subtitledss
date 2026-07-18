import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface DiarizedSegment {
  start: number;
  end: number;
  text: string;
  speaker: string | null;
}

export interface VideoTranscriptionResult {
  id: number;
  video_name: string;
  segments: DiarizedSegment[];
  full_text: string;
  translated_text: string | null;
  target_language: string | null;
  duration_seconds: number;
}

export interface VideoTranscriptionEntry {
  id: number;
  video_path: string;
  video_name: string;
  duration_seconds: number | null;
  language: string;
  full_text: string;
  translated_text: string | null;
  target_language: string | null;
  summary: string | null;
  segments: DiarizedSegment[];
  created_at: string;
}

export type TranscriptionStep =
  | "idle"
  | "extracting"
  | "diarizing"
  | "transcribing"
  | "translating"
  | "merging"
  | "saving"
  | "done"
  | "error";

interface ProgressInfo {
  step: TranscriptionStep;
  progress: number;
  message: string;
}

export function useVideoTranscription() {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [result, setResult] = useState<VideoTranscriptionResult | null>(null);
  const [step, setStep] = useState<TranscriptionStep>("idle");
  const [progress, setProgress] = useState<ProgressInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<VideoTranscriptionEntry[]>([]);

  useEffect(() => {
    const unlisten = listen<ProgressInfo>("video-transcription-progress", (event) => {
      setProgress(event.payload);
      setStep(event.payload.step);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadHistory = useCallback(async () => {
    try {
      const entries = await invoke<VideoTranscriptionEntry[]>("list_video_transcriptions", {
        limit: 50,
      });
      setHistory(entries);
    } catch (err) {
      console.error("Failed to load video transcriptions:", err);
    }
  }, []);

  const transcribe = useCallback(
    async (
      videoPath: string,
      language?: string,
      enableDiarization?: boolean,
      targetLanguage?: string,
    ) => {
      try {
        setError(null);
        setResult(null);
        setStep("extracting");

        const result = await invoke<VideoTranscriptionResult>("transcribe_video", {
          videoPath,
          language: language || null,
          enableDiarization: enableDiarization || false,
          targetLanguage: targetLanguage || null,
        });

        setResult(result);
        setStep("done");
        loadHistory();
        return result;
      } catch (err) {
        const msg = typeof err === "string" ? err : "Transcription failed";
        setError(msg);
        setStep("error");
        throw err;
      }
    },
    [loadHistory],
  );

  const deleteTranscription = useCallback(
    async (id: number) => {
      try {
        await invoke("delete_video_transcription", { id });
        loadHistory();
      } catch (err) {
        console.error("Failed to delete transcription:", err);
      }
    },
    [loadHistory],
  );

  const exportTranscription = useCallback(async (id: number, format: string) => {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const ext = format === "ass" ? ".ass" : `.${format}`;
      const path = await save({
        defaultPath: `transcription${ext}`,
        filters: [{ name: `${format.toUpperCase()} files`, extensions: [format] }],
      });
      if (path) {
        await invoke("export_video_transcription", { id, format, path });
      }
    } catch (err) {
      console.error("Failed to export:", err);
      throw err;
    }
  }, []);

  return {
    selectedFile,
    setSelectedFile,
    result,
    step,
    progress,
    error,
    history,
    transcribe,
    deleteTranscription,
    exportTranscription,
    loadHistory,
  };
}
