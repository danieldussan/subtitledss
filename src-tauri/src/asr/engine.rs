use std::path::{Path, PathBuf};

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
}

impl EngineKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EngineKind::Whisper => "whisper",
            EngineKind::Sherpa => "sherpa",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "sherpa" => EngineKind::Sherpa,
            _ => EngineKind::Whisper,
        }
    }
}

/// Unified ASR engine. Wraps either whisper.cpp (WhisperEngine) or
/// sherpa-onnx (SherpaEngine) behind a single API so the rest of the app
/// does not need to know which backend is active.
pub enum AsrEngine {
    Whisper(WhisperEngine),
    Sherpa(SherpaEngine),
}

impl AsrEngine {
    pub fn new(kind: EngineKind) -> Self {
        match kind {
            EngineKind::Whisper => AsrEngine::Whisper(WhisperEngine::new()),
            EngineKind::Sherpa => AsrEngine::Sherpa(SherpaEngine::new()),
        }
    }

    pub fn kind(&self) -> EngineKind {
        match self {
            AsrEngine::Whisper(_) => EngineKind::Whisper,
            AsrEngine::Sherpa(_) => EngineKind::Sherpa,
        }
    }

    pub fn load_model(&mut self, model_path: &Path) -> anyhow::Result<()> {
        match self {
            AsrEngine::Whisper(e) => e.load_model(model_path),
            AsrEngine::Sherpa(e) => e.load_model(model_path),
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
        }
    }

    pub fn is_loaded(&self) -> bool {
        match self {
            AsrEngine::Whisper(e) => e.is_loaded(),
            AsrEngine::Sherpa(e) => e.is_loaded(),
        }
    }

    pub fn model_path(&self) -> Option<PathBuf> {
        match self {
            AsrEngine::Whisper(e) => e.model_path().map(PathBuf::clone),
            AsrEngine::Sherpa(e) => e.model_path(),
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
        }
    }
}
