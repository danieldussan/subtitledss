import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface AppConfig {
  audio: {
    source: string;
    device: string;
    sample_rate: number;
    vad_threshold: number;
  };
  whisper: {
    model: string;
    language: string;
    threads: number;
    gpu: boolean;
  };
  overlay: {
    x: number;
    y: number;
    width: number;
    height: number;
    opacity: number;
    always_on_top: boolean;
    click_through: boolean;
    font_size: number;
    font_color: string;
    background_color: string;
    auto_hide: boolean;
    auto_hide_delay: number;
    display_duration_ms: number;
    fade_duration_ms: number;
    max_visible_lines: number;
    line_gap: number;
    max_line_width: number;
  };
  translation: {
    enabled: boolean;
    source_lang: string;
    target_lang: string;
    show_original: boolean;
  };
  shortcuts: {
    toggle_capture: string;
    toggle_overlay: string;
    toggle_translation: string;
    clear_history: string;
  };
  ai: {
    provider: string;
    base_url: string;
    api_key: string | null;
    model: string;
  };
  onboarding_completed: boolean;
}

export function useSettings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const cfg = await invoke<AppConfig>("get_config");
      setConfig(cfg);
      setError(null);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = useCallback(async (newConfig: AppConfig) => {
    try {
      await invoke("save_config", { config: newConfig });
      setConfig(newConfig);
      setError(null);
    } catch (err) {
      setError(err as string);
    }
  }, []);

  const updateConfig = useCallback(
    (updates: Partial<AppConfig>) => {
      if (!config) return;
      const newConfig = { ...config, ...updates };
      saveConfig(newConfig);
    },
    [config, saveConfig],
  );

  return { config, loading, error, saveConfig, updateConfig, reload: loadConfig };
}
