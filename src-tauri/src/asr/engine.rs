use std::path::{Path, PathBuf};

use crate::ct2::engine::Ctranslate2Engine;
use crate::sherpa::engine::SherpaEngine;
use crate::whisper::engine::WhisperEngine;
use crate::whisper::params::TranscriptionParams;

#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    Whisper,
    Sherpa,
    Ctranslate2,
}

impl EngineKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EngineKind::Whisper => "whisper",
            EngineKind::Sherpa => "sherpa",
            EngineKind::Ctranslate2 => "ctranslate2",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "sherpa" => EngineKind::Sherpa,
            "ct2" | "ctranslate2" => EngineKind::Ctranslate2,
            _ => EngineKind::Whisper,
        }
    }
}

/// Unified ASR engine. Wraps whisper.cpp (WhisperEngine), sherpa-onnx
/// (SherpaEngine) or CTranslate2 (Ctranslate2Engine) behind a single API so
/// the rest of the app does not need to know which backend is active.
pub enum AsrEngine {
    Whisper(WhisperEngine),
    Sherpa(SherpaEngine),
    Ctranslate2(Ctranslate2Engine),
}

impl AsrEngine {
    pub fn new(kind: EngineKind) -> Self {
        match kind {
            EngineKind::Whisper => AsrEngine::Whisper(WhisperEngine::new()),
            EngineKind::Sherpa => AsrEngine::Sherpa(SherpaEngine::new()),
            EngineKind::Ctranslate2 => AsrEngine::Ctranslate2(Ctranslate2Engine::new()),
        }
    }

    pub fn kind(&self) -> EngineKind {
        match self {
            AsrEngine::Whisper(_) => EngineKind::Whisper,
            AsrEngine::Sherpa(_) => EngineKind::Sherpa,
            AsrEngine::Ctranslate2(_) => EngineKind::Ctranslate2,
        }
    }

    /// Replace the inner engine with a new one of the given kind if it doesn't
    /// already match. Returns `true` if the engine was swapped.
    pub fn switch_kind(&mut self, kind: EngineKind) -> bool {
        if self.kind() != kind {
            *self = AsrEngine::new(kind);
            true
        } else {
            false
        }
    }

    pub fn load_model(&mut self, model_path: &Path, gpu: bool) -> anyhow::Result<()> {
        match self {
            AsrEngine::Whisper(e) => e.load_model(model_path, gpu),
            AsrEngine::Sherpa(e) => e.load_model(model_path, gpu),
            AsrEngine::Ctranslate2(e) => e.load_model(model_path, gpu),
        }
    }

    /// Forward the desired compute type to the CTranslate2 engine (no-op for
    /// the other backends).
    pub fn set_compute_type(&mut self, compute_type: &str) {
        if let AsrEngine::Ctranslate2(e) = self {
            e.set_compute_type(compute_type.to_string());
        }
    }

    pub fn transcribe(
        &mut self,
        audio: &[f32],
        params: &TranscriptionParams,
    ) -> anyhow::Result<Vec<TranscriptionSegment>> {
        match self {
            AsrEngine::Whisper(e) => e.transcribe(audio, params),
            AsrEngine::Sherpa(e) => e.transcribe(audio, params),
            AsrEngine::Ctranslate2(e) => e.transcribe(audio, params),
        }
    }

    pub fn is_loaded(&self) -> bool {
        match self {
            AsrEngine::Whisper(e) => e.is_loaded(),
            AsrEngine::Sherpa(e) => e.is_loaded(),
            AsrEngine::Ctranslate2(e) => e.is_loaded(),
        }
    }

    pub fn model_path(&self) -> Option<PathBuf> {
        match self {
            AsrEngine::Whisper(e) => e.model_path().map(PathBuf::clone),
            AsrEngine::Sherpa(e) => e.model_path(),
            AsrEngine::Ctranslate2(e) => e.model_path(),
        }
    }

    pub fn model_name(&self) -> Option<String> {
        match self {
            AsrEngine::Whisper(e) => e.model_path().and_then(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.strip_prefix("ggml-").unwrap_or(s).to_string())
            }),
            AsrEngine::Sherpa(e) => e.model_name(),
            AsrEngine::Ctranslate2(e) => e.model_name(),
        }
    }
}
