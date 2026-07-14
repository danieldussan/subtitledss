pub mod marian;
pub mod model;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub source_lang: String,
    pub target_lang: String,
    pub show_original: bool,
}

/// Translate text using Marian MT (in-process, offline).
/// Returns Ok(translated_text) or Err(message).
pub async fn translate(
    text: &str,
    config: &TranslationConfig,
    marian_engine: &std::sync::Arc<std::sync::Mutex<marian::MarianEngine>>,
) -> anyhow::Result<String> {
    if !config.enabled {
        return Ok(text.to_string());
    }

    // Skip translation if source and target are the same
    if config.source_lang == config.target_lang {
        return Ok(text.to_string());
    }

    // Marian is synchronous — run in spawn_blocking to avoid blocking the async runtime
    let text = text.to_string();
    let source_lang = config.source_lang.clone();
    let target_lang = config.target_lang.clone();
    let engine = marian_engine.clone();

    tokio::task::spawn_blocking(move || {
        let mut eng = engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Engine lock poisoned: {}", e))?;
        eng.load(&source_lang, &target_lang)?;
        eng.translate(&text)
    })
    .await?
}
