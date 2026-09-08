use std::path::{Path, PathBuf};

use sherpa_onnx::{
    OfflineCanaryModelConfig, OfflineRecognizer, OfflineRecognizerConfig,
    OfflineSenseVoiceModelConfig, OfflineTransducerModelConfig,
};
use tracing::{info, warn};

use crate::asr::engine::TranscriptionSegment;
use crate::whisper::params::TranscriptionParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SherpaModelKind {
    SenseVoice,
    ParakeetTdt,
    Canary,
}

impl SherpaModelKind {
    /// Infer the model kind from the model directory name (the sherpa-onnx
    /// packages keep a stable, descriptive name).
    pub fn from_dir_name(name: &str) -> Option<Self> {
        if name.contains("sense-voice") {
            Some(Self::SenseVoice)
        } else if name.contains("parakeet-tdt") {
            Some(Self::ParakeetTdt)
        } else if name.contains("canary") {
            Some(Self::Canary)
        } else {
            None
        }
    }
}

/// sherpa-onnx backed engine. Mirrors the `WhisperEngine::transcribe` API so
/// it can be swapped in through the `AsrEngine` wrapper.
pub struct SherpaEngine {
    recognizer: Option<OfflineRecognizer>,
    model_path: Option<PathBuf>,
    model_name: Option<String>,
    kind: Option<SherpaModelKind>,
    threads: i32,
    provider: String,
    /// Canary encodes src/tgt language into the graph; we rebuild the
    /// recognizer if the requested language changes.
    canary_lang: Option<String>,
}

impl SherpaEngine {
    pub fn new() -> Self {
        Self {
            recognizer: None,
            model_path: None,
            model_name: None,
            kind: None,
            threads: 4,
            provider: "cpu".to_string(),
            canary_lang: None,
        }
    }

    /// Load a sherpa-onnx model package. `model_path` must point to a
    /// directory containing the ONNX files + tokens.txt (e.g.
    /// `models/sherpa/parakeet-tdt-0.6b-v3-int8/`).
    pub fn load_model(&mut self, model_path: &Path, gpu: bool) -> anyhow::Result<()> {
        if !model_path.exists() || !model_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Sherpa model directory not found: {:?}",
                model_path
            ));
        }

        let name = model_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid model path: {:?}", model_path))?;

        let kind = SherpaModelKind::from_dir_name(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown sherpa model type in '{}'", name))?;

        let provider = if gpu { "cuda" } else { "cpu" };
        info!(
            "Loading sherpa-onnx model '{}' ({:?}, provider={})",
            name, kind, provider
        );

        // Try requested provider; fall back to CPU if unavailable
        let mut actual_provider = provider.to_string();
        let recognizer = build_recognizer(model_path, kind, self.threads, None, provider)
            .or_else(|e| {
                if provider == "cuda" {
                    warn!("CUDA provider unavailable ({}), falling back to CPU", e);
                    actual_provider = "cpu".to_string();
                    build_recognizer(model_path, kind, self.threads, None, "cpu")
                } else {
                    Err(e)
                }
            })
            .map_err(|e| {
                anyhow::anyhow!("Failed to create sherpa recognizer for '{}': {}", name, e)
            })?;

        self.recognizer = Some(recognizer);
        self.model_path = Some(model_path.to_path_buf());
        self.model_name = Some(name.to_string());
        self.kind = Some(kind);
        self.provider = actual_provider;
        self.canary_lang = None;

        info!("Sherpa model '{}' loaded", name);
        Ok(())
    }

    pub fn transcribe(
        &mut self,
        audio: &[f32],
        params: &TranscriptionParams,
    ) -> anyhow::Result<Vec<TranscriptionSegment>> {
        let Some(kind) = self.kind else {
            return Err(anyhow::anyhow!("Sherpa model not loaded"));
        };

        // Canary encodes the language pair into the graph, so rebuild the
        // recognizer when the requested language changes (rare at runtime).
        if kind == SherpaModelKind::Canary {
            let lang = resolve_canary_lang(params.language.as_deref());
            if self.canary_lang.as_deref() != Some(lang.as_str()) {
                let model_path = self
                    .model_path
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Sherpa model not loaded"))?;
                info!("Rebuilding Canary recognizer for language '{}'", lang);
                let recognizer =
                    build_recognizer(model_path, kind, self.threads, Some(&lang), &self.provider)
                        .map_err(|e| anyhow::anyhow!("Failed to reload Canary recognizer: {}", e))?;
                self.recognizer = Some(recognizer);
                self.canary_lang = Some(lang);
            }
        }

        let recognizer = self
            .recognizer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Sherpa model not loaded"))?;

        let stream = recognizer.create_stream();
        stream.accept_waveform(16000, audio);
        recognizer.decode(&stream);

        let result = stream
            .get_result()
            .ok_or_else(|| anyhow::anyhow!("Sherpa decode produced no result"))?;

        let text = result.text.trim().to_string();
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let duration = audio.len() as f64 / 16000.0;

        // Prefer per-token timestamps when the model emits them so subtitle
        // segments get real boundaries; otherwise fall back to one segment.
        if let Some(timestamps) = result.timestamps {
            let tokens = result.tokens;
            if !timestamps.is_empty() && tokens.len() == timestamps.len() {
                let mut segments = Vec::new();
                let mut start = 0.0;
                let mut buffer = String::new();
                for (i, t) in timestamps.iter().enumerate() {
                    if buffer.is_empty() {
                        start = *t as f64;
                    }
                    buffer.push_str(&tokens[i]);
                    if tokens[i].ends_with(' ') {
                        let trimmed = buffer.trim().to_string();
                        if !trimmed.is_empty() {
                            segments.push(TranscriptionSegment {
                                start,
                                end: *t as f64,
                                text: trimmed,
                            });
                        }
                        buffer.clear();
                    }
                }
                let trimmed = buffer.trim().to_string();
                if !trimmed.is_empty() {
                    segments.push(TranscriptionSegment {
                        start,
                        end: duration,
                        text: trimmed,
                    });
                }
                if !segments.is_empty() {
                    return Ok(segments);
                }
            }
        }

        Ok(vec![TranscriptionSegment {
            start: 0.0,
            end: duration,
            text,
        }])
    }

    pub fn is_loaded(&self) -> bool {
        self.recognizer.is_some()
    }

    pub fn model_path(&self) -> Option<PathBuf> {
        self.model_path.clone()
    }

    pub fn model_name(&self) -> Option<String> {
        self.model_name.clone()
    }

    pub fn set_threads(&mut self, threads: i32) {
        self.threads = threads;
    }
}

impl Default for SherpaEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// sherpa-onnx's Canary runtime currently validates src/tgt languages against
/// en/de/es/fr, so map anything else to "en".
fn resolve_canary_lang(lang: Option<&str>) -> String {
    match lang {
        Some("es") => "es".to_string(),
        Some("en") => "en".to_string(),
        Some("de") => "de".to_string(),
        Some("fr") => "fr".to_string(),
        _ => "en".to_string(),
    }
}

fn build_recognizer(
    dir: &Path,
    kind: SherpaModelKind,
    threads: i32,
    canary_lang: Option<&str>,
    provider: &str,
) -> anyhow::Result<OfflineRecognizer> {
    let mut config = OfflineRecognizerConfig::default();
    config.model_config.num_threads = threads;
    config.model_config.provider = Some(provider.to_string());

    match kind {
        SherpaModelKind::SenseVoice => {
            config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
                model: Some(dir.join("model.int8.onnx").to_string_lossy().into_owned()),
                language: Some("auto".to_string()),
                use_itn: true,
            };
            config.model_config.tokens =
                Some(dir.join("tokens.txt").to_string_lossy().into_owned());
        }
        SherpaModelKind::ParakeetTdt => {
            config.model_config.transducer = OfflineTransducerModelConfig {
                encoder: Some(dir.join("encoder.int8.onnx").to_string_lossy().into_owned()),
                decoder: Some(dir.join("decoder.int8.onnx").to_string_lossy().into_owned()),
                joiner: Some(dir.join("joiner.int8.onnx").to_string_lossy().into_owned()),
            };
            config.model_config.tokens =
                Some(dir.join("tokens.txt").to_string_lossy().into_owned());
            config.model_config.model_type = Some("nemo_transducer".to_string());
        }
        SherpaModelKind::Canary => {
            let lang = canary_lang.unwrap_or("en").to_string();
            config.model_config.canary = OfflineCanaryModelConfig {
                encoder: Some(dir.join("encoder.int8.onnx").to_string_lossy().into_owned()),
                decoder: Some(dir.join("decoder.int8.onnx").to_string_lossy().into_owned()),
                src_lang: Some(lang.clone()),
                tgt_lang: Some(lang),
                use_pnc: true,
            };
            config.model_config.tokens =
                Some(dir.join("tokens.txt").to_string_lossy().into_owned());
        }
    }

    OfflineRecognizer::create(&config).ok_or_else(|| {
        anyhow::anyhow!(
            "OfflineRecognizer::create returned None (provider={})",
            provider
        )
    })
}
