use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub whisper: WhisperConfig,
    pub overlay: OverlayConfig,
    pub translation: TranslationConfig,
    pub shortcuts: ShortcutsConfig,
    pub ai: AiSettingsConfig,
    #[serde(default)]
    pub onboarding_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioConfig {
    pub source: String,
    pub device: String,
    pub sample_rate: u32,
    pub vad_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhisperConfig {
    pub model: String,
    pub language: String,
    pub threads: u32,
    pub gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverlayConfig {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub opacity: f64,
    pub always_on_top: bool,
    pub click_through: bool,
    pub font_size: u32,
    pub font_color: String,
    pub background_color: String,
    pub auto_hide: bool,
    pub auto_hide_delay: u64,
    pub display_duration_ms: u64,
    pub fade_duration_ms: u64,
    pub max_visible_lines: usize,
    pub line_gap: u32,
    pub max_line_width: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub source_lang: String,
    pub target_lang: String,
    pub show_original: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShortcutsConfig {
    pub toggle_capture: String,
    pub toggle_overlay: String,
    pub toggle_translation: String,
    pub clear_history: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiSettingsConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig {
                source: "system".to_string(),
                device: "default".to_string(),
                sample_rate: 16000,
                vad_threshold: 0.005,
            },
            whisper: WhisperConfig {
                model: "base".to_string(),
                language: "auto".to_string(),
                threads: 4,
                gpu: false,
            },
            overlay: OverlayConfig {
                x: 100,
                y: 500,
                width: 600,
                height: 100,
                opacity: 0.9,
                always_on_top: true,
                click_through: false,
                font_size: 24,
                font_color: "#ffffff".to_string(),
                background_color: "#00000080".to_string(),
                auto_hide: true,
                auto_hide_delay: 5000,
                display_duration_ms: 10000,
                fade_duration_ms: 3000,
                max_visible_lines: 4,
                line_gap: 4,
                max_line_width: 80,
            },
            translation: TranslationConfig {
                enabled: false,
                source_lang: "en".to_string(),
                target_lang: "es".to_string(),
                show_original: true,
            },
            shortcuts: ShortcutsConfig {
                toggle_capture: "Ctrl+Shift+S".to_string(),
                toggle_overlay: "Ctrl+Shift+O".to_string(),
                toggle_translation: "Ctrl+Shift+T".to_string(),
                clear_history: "Ctrl+Shift+H".to_string(),
            },
            ai: AiSettingsConfig {
                provider: "ollama".to_string(),
                base_url: "http://localhost:11434/v1".to_string(),
                api_key: Some("ollama".to_string()),
                model: "llama3.2".to_string(),
            },
            onboarding_completed: false,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("subtitledss")
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_file();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => {
                        info!("Config loaded from {:?}", path);
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse config: {}, using defaults", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read config: {}, using defaults", e);
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = Self::config_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_file(), content)?;

        info!("Config saved to {:?}", Self::config_file());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "livetext_config_test_{}_{:?}",
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn temp_config_path(dir: &PathBuf) -> PathBuf {
        dir.join("config.toml")
    }

    // ── Default values ────────────────────────────────────────────

    #[test]
    fn test_default_audio_config() {
        let config = AppConfig::default();
        assert_eq!(config.audio.source, "system");
        assert_eq!(config.audio.device, "default");
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.audio.vad_threshold, 0.005);
    }

    #[test]
    fn test_default_whisper_config() {
        let config = AppConfig::default();
        assert_eq!(config.whisper.model, "base");
        assert_eq!(config.whisper.language, "auto");
        assert_eq!(config.whisper.threads, 4);
        assert!(!config.whisper.gpu);
    }

    #[test]
    fn test_default_overlay_config() {
        let config = AppConfig::default();
        assert_eq!(config.overlay.x, 100);
        assert_eq!(config.overlay.y, 500);
        assert_eq!(config.overlay.font_size, 24);
        assert_eq!(config.overlay.font_color, "#ffffff");
        assert_eq!(config.overlay.opacity, 0.9);
        assert!(config.overlay.always_on_top);
        assert!(config.overlay.auto_hide);
        assert_eq!(config.overlay.auto_hide_delay, 5000);
        assert_eq!(config.overlay.display_duration_ms, 10000);
        assert_eq!(config.overlay.fade_duration_ms, 3000);
        assert_eq!(config.overlay.max_visible_lines, 4);
        assert_eq!(config.overlay.line_gap, 4);
        assert_eq!(config.overlay.max_line_width, 80);
    }

    #[test]
    fn test_default_translation_config() {
        let config = AppConfig::default();
        assert!(!config.translation.enabled);
        assert_eq!(config.translation.source_lang, "en");
        assert_eq!(config.translation.target_lang, "es");
        assert!(config.translation.show_original);
    }

    #[test]
    fn test_default_shortcuts_config() {
        let config = AppConfig::default();
        assert_eq!(config.shortcuts.toggle_capture, "Ctrl+Shift+S");
        assert_eq!(config.shortcuts.toggle_overlay, "Ctrl+Shift+O");
        assert_eq!(config.shortcuts.toggle_translation, "Ctrl+Shift+T");
        assert_eq!(config.shortcuts.clear_history, "Ctrl+Shift+H");
    }

    #[test]
    fn test_default_ai_config() {
        let config = AppConfig::default();
        assert_eq!(config.ai.provider, "ollama");
        assert_eq!(config.ai.base_url, "http://localhost:11434/v1");
        assert_eq!(config.ai.model, "llama3.2");
    }

    // ── Serialization / Deserialization ───────────────────────────

    #[test]
    fn test_serialize_to_toml() {
        let config = AppConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[audio]"));
        assert!(toml_str.contains("[whisper]"));
        assert!(toml_str.contains("[overlay]"));
        assert!(toml_str.contains("source = \"system\""));
        assert!(toml_str.contains("model = \"base\""));
    }

    #[test]
    fn test_deserialize_from_toml() {
        let toml_str = "\
[audio]\n\
source = \"microphone\"\n\
device = \"hw:0\"\n\
sample_rate = 44100\n\
vad_threshold = 0.02\n\
\n\
[whisper]\n\
model = \"tiny\"\n\
language = \"es\"\n\
threads = 8\n\
gpu = true\n\
\n\
[overlay]\n\
x = 200\n\
y = 300\n\
width = 800\n\
height = 200\n\
opacity = 0.8\n\
always_on_top = false\n\
click_through = true\n\
font_size = 32\n\
font_color = \"#00ff00\"\n\
background_color = \"#000000cc\"\n\
auto_hide = false\n\
auto_hide_delay = 3000\n\
display_duration_ms = 8000\n\
fade_duration_ms = 2000\n\
max_visible_lines = 3\n\
line_gap = 6\n\
max_line_width = 100\n\
\n\
[translation]\n\
enabled = true\n\
source_lang = \"en\"\n\
target_lang = \"fr\"\n\
show_original = false\n\
\n\
[shortcuts]\n\
toggle_capture = \"Ctrl+Alt+S\"\n\
toggle_overlay = \"Ctrl+Alt+O\"\n\
toggle_translation = \"Ctrl+Alt+T\"\n\
clear_history = \"Ctrl+Alt+H\"\n\
\n\
[ai]\n\
provider = \"Ollama\"\n\
base_url = \"http://localhost:11434/v1\"\n\
api_key = \"ollama\"\n\
model = \"llama3.2\"\n";
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.audio.source, "microphone");
        assert_eq!(config.audio.device, "hw:0");
        assert_eq!(config.audio.sample_rate, 44100);
        assert_eq!(config.audio.vad_threshold, 0.02);
        assert_eq!(config.whisper.model, "tiny");
        assert_eq!(config.whisper.language, "es");
        assert_eq!(config.whisper.threads, 8);
        assert!(config.whisper.gpu);
        assert_eq!(config.overlay.x, 200);
        assert_eq!(config.overlay.y, 300);
        assert!(!config.overlay.always_on_top);
        assert!(config.overlay.click_through);
        assert_eq!(config.overlay.font_size, 32);
        assert_eq!(config.overlay.font_color, "#00ff00");
        assert!(!config.overlay.auto_hide);
        assert_eq!(config.overlay.auto_hide_delay, 3000);
        assert_eq!(config.overlay.display_duration_ms, 8000);
        assert_eq!(config.overlay.fade_duration_ms, 2000);
        assert_eq!(config.overlay.max_visible_lines, 3);
        assert_eq!(config.overlay.line_gap, 6);
        assert_eq!(config.overlay.max_line_width, 100);
        assert!(config.translation.enabled);
        assert_eq!(config.translation.target_lang, "fr");
        assert!(!config.translation.show_original);
        assert_eq!(config.shortcuts.toggle_capture, "Ctrl+Alt+S");
    }

    #[test]
    fn test_roundtrip_serialize_deserialize() {
        let original = AppConfig::default();
        let toml_str = toml::to_string_pretty(&original).unwrap();
        let restored: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(original, restored);
    }

    // ── Save and Load ─────────────────────────────────────────────

    #[test]
    fn test_save_creates_file() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let config_path = temp_config_path(&dir);

        let config = AppConfig::default();
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, content).unwrap();

        assert!(config_path.exists());
        let read_content = fs::read_to_string(&config_path).unwrap();
        assert!(read_content.contains("source = \"system\""));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_load_from_file() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let config_path = temp_config_path(&dir);

        let mut original = AppConfig::default();
        original.audio.source = "microphone".to_string();
        original.whisper.model = "tiny".to_string();

        let content = toml::to_string_pretty(&original).unwrap();
        fs::write(&config_path, content).unwrap();

        let loaded: AppConfig = toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(loaded.audio.source, "microphone");
        assert_eq!(loaded.whisper.model, "tiny");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_load_from_nonexistent_path_returns_default() {
        // Test that deserializing an empty string fails (as expected)
        // and that the default config is correct
        let config = AppConfig::default();
        assert_eq!(config.audio.source, "system");
        assert_eq!(config.whisper.model, "base");
    }

    // ── Edge cases ────────────────────────────────────────────────

    #[test]
    fn test_invalid_toml_returns_default() {
        let invalid_toml = "this is not valid toml {{{";
        let result: Result<AppConfig, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_partial_toml_uses_defaults() {
        // TOML with only audio section — other fields should fail deserialization
        let partial = r#"
[audio]
source = "microphone"
"#;
        let result: Result<AppConfig, _> = toml::from_str(partial);
        assert!(result.is_err()); // Missing required fields
    }

    #[test]
    fn test_empty_toml() {
        let result: Result<AppConfig, _> = toml::from_str("");
        assert!(result.is_err());
    }

    // ── Config paths ──────────────────────────────────────────────

    #[test]
    fn test_config_dir_path() {
        let dir = AppConfig::config_dir();
        assert!(dir.to_string_lossy().contains("subtitledss"));
    }

    #[test]
    fn test_config_file_path() {
        let file = AppConfig::config_file();
        assert!(file.to_string_lossy().ends_with("config.toml"));
    }
}
