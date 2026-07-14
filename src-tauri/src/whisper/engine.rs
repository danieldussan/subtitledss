use std::path::PathBuf;
use std::sync::Arc;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use tracing::info;

use super::params::TranscriptionParams;

pub struct WhisperEngine {
    context: Option<Arc<WhisperContext>>,
    model_path: Option<PathBuf>,
}

impl WhisperEngine {
    pub fn new() -> Self {
        Self {
            context: None,
            model_path: None,
        }
    }

    pub fn load_model(&mut self, model_path: &PathBuf) -> anyhow::Result<()> {
        info!("Loading Whisper model from {:?}", model_path);

        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model file not found: {:?}", model_path));
        }

        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to load model: {}", e))?;

        self.context = Some(Arc::new(ctx));
        self.model_path = Some(model_path.clone());

        info!("Whisper model loaded successfully from {:?}", model_path);
        Ok(())
    }

    pub fn transcribe(
        &self,
        audio_data: &[f32],
        params: &TranscriptionParams,
    ) -> anyhow::Result<Vec<TranscriptionSegment>> {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model not loaded"))?;

        let mut whisper_params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        if let Some(lang) = &params.language {
            whisper_params.set_language(Some(lang));
        }

        whisper_params.set_n_threads(params.threads as i32);
        whisper_params.set_print_special(false);
        whisper_params.set_print_progress(false);
        whisper_params.set_print_realtime(false);
        whisper_params.set_print_timestamps(false);
        whisper_params.set_suppress_blank(true);
        whisper_params.set_translate(params.translate);

        let mut state = context.create_state()?;

        state
            .full(whisper_params, audio_data)
            .map_err(|e| anyhow::anyhow!("Transcription failed: {}", e))?;

        let mut segments = Vec::new();
        for segment in state.as_iter() {
            segments.push(TranscriptionSegment {
                start: segment.start_timestamp() as f64 / 100.0,
                end: segment.end_timestamp() as f64 / 100.0,
                text: segment.to_string(),
            });
        }

        Ok(segments)
    }

    pub fn is_loaded(&self) -> bool {
        self.context.is_some()
    }

    pub fn model_path(&self) -> Option<&PathBuf> {
        self.model_path.as_ref()
    }
}

impl Default for WhisperEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}
