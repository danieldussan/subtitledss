use std::path::PathBuf;
use candle_core::{DType, Device, Tensor, D};
use candle_nn::VarBuilder;
use candle_transformers::models::marian;
use tokenizers::Tokenizer;
use tracing::info;

use super::model::{MarianModelInfo, MarianModelManager};

/// Direction of translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationDirection {
    EnToEs,
    EsToEn,
}

impl TranslationDirection {
    pub fn from_langs(source: &str, target: &str) -> Option<Self> {
        match (source, target) {
            ("en", "es") => Some(Self::EnToEs),
            ("es", "en") => Some(Self::EsToEn),
            _ => None,
        }
    }

    pub fn pair_key(&self) -> &str {
        match self {
            Self::EnToEs => "en-es",
            Self::EsToEn => "es-en",
        }
    }
}

/// Marian translation engine — runs entirely offline on CPU.
pub struct MarianEngine {
    manager: MarianModelManager,
    loaded_pair: Option<TranslationDirection>,
    model_data: Option<LoadedModel>,
}

struct LoadedModel {
    model: marian::MTModel,
    src_tokenizer: Tokenizer,
    tgt_tokenizer: Tokenizer,
    config: marian::Config,
}

impl MarianEngine {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            manager: MarianModelManager::new(models_dir),
            loaded_pair: None,
            model_data: None,
        }
    }

    /// Get the models directory path.
    pub fn models_dir(&self) -> PathBuf {
        self.manager.models_dir()
    }

    /// Check if a language pair is supported.
    pub fn is_supported(source: &str, target: &str) -> bool {
        TranslationDirection::from_langs(source, target).is_some()
    }

    /// Check if model files are downloaded for a pair.
    pub fn is_downloaded(&self, source: &str, target: &str) -> bool {
        match TranslationDirection::from_langs(source, target) {
            Some(dir) => self.manager.is_downloaded(dir.pair_key()),
            None => false,
        }
    }

    /// Download model files for a language pair (async).
    pub async fn download_async(&self, source: &str, target: &str) -> anyhow::Result<()> {
        let dir = TranslationDirection::from_langs(source, target)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language pair: {} → {}", source, target))?;
        let info = match dir {
            TranslationDirection::EnToEs => MarianModelInfo::en_es(),
            TranslationDirection::EsToEn => MarianModelInfo::es_en(),
        };
        self.manager.download_async(&info).await
    }

    /// Delete model files for a language pair.
    pub fn delete_model(&self, source: &str, target: &str) -> anyhow::Result<()> {
        let dir = TranslationDirection::from_langs(source, target)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language pair: {} → {}", source, target))?;
        self.manager.delete(dir.pair_key())
    }

    /// Load the model for a language pair. If already loaded for a different pair, reload.
    pub fn load(&mut self, source: &str, target: &str) -> anyhow::Result<()> {
        let dir = TranslationDirection::from_langs(source, target)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language pair: {} → {}", source, target))?;

        // Skip if already loaded for this pair
        if self.loaded_pair == Some(dir) && self.model_data.is_some() {
            return Ok(());
        }

        // Ensure model is downloaded
        if !self.manager.is_downloaded(dir.pair_key()) {
            // Use blocking runtime for sync context
            let rt = tokio::runtime::Handle::current();
            let info = match dir {
                TranslationDirection::EnToEs => MarianModelInfo::en_es(),
                TranslationDirection::EsToEn => MarianModelInfo::es_en(),
            };
            let manager = &self.manager;
            tokio::task::block_in_place(|| {
                rt.block_on(manager.download_async(&info))
            })?;
        }

        let device = Device::Cpu;

        // Build config
        let config = match dir {
            TranslationDirection::EnToEs => marian::Config::opus_mt_en_es(),
            TranslationDirection::EsToEn => {
                match self.build_config_from_repo() {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Failed to load ES→EN config from repo ({}), using hardcoded fallback", e);
                        Self::opus_mt_es_en()
                    }
                }
            }
        };

        // Load tokenizers
        let src_tokenizer = Tokenizer::from_file(self.manager.src_tokenizer_path(dir.pair_key()))
            .map_err(|e| anyhow::anyhow!("Failed to load source tokenizer: {}", e))?;
        let tgt_tokenizer = Tokenizer::from_file(self.manager.tgt_tokenizer_path(dir.pair_key()))
            .map_err(|e| anyhow::anyhow!("Failed to load target tokenizer: {}", e))?;

        // Load model weights
        let model_path = self.manager.model_path(dir.pair_key());
        info!("Loading Marian model from {:?}", model_path);
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path.to_str().unwrap()], DType::F32, &device)?
        };
        let model = marian::MTModel::new(&config, vb)?;

        self.model_data = Some(LoadedModel {
            model,
            src_tokenizer,
            tgt_tokenizer,
            config,
        });
        self.loaded_pair = Some(dir);

        info!("Marian model loaded for {}", dir.pair_key());
        Ok(())
    }

    /// Translate text. Returns the translated string.
    pub fn translate(&mut self, text: &str) -> anyhow::Result<String> {
        let loaded = self
            .model_data
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("No Marian model loaded. Call load() first."))?;

        let config = &loaded.config;
        let max_len = 512;

        // Tokenize input
        let encoding = loaded
            .src_tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        let mut token_ids: Vec<u32> = encoding.get_ids().to_vec();
        token_ids.push(config.eos_token_id);

        let device = Device::Cpu;
        let tokens = Tensor::new(token_ids.as_slice(), &device)?.unsqueeze(0)?;

        // Reset encoder KV cache (encoder attention is incorrectly created
        // with is_decoder=true in candle, so it caches across calls)
        loaded.model.encoder().reset_kv_cache();

        // Encode
        let encoder_xs = loaded.model.encoder().forward(&tokens, 0)?;

        // Reset decoder KV cache before decode loop
        loaded.model.decoder().reset_kv_cache();

        // Streaming decode: feed one token at a time with correct past_kv_len
        let mut output_ids = vec![config.decoder_start_token_id];
        for step in 0..max_len {
            // Feed only the last token (streaming decode)
            let last_id = *output_ids.last().unwrap();
            let input_ids = Tensor::new(&[last_id], &device)?.unsqueeze(0)?;
            let logits = loaded.model.decode(&input_ids, &encoder_xs, step)?;
            let logits = logits.squeeze(0)?;
            let logits = logits.get(logits.dim(0)? - 1)?;
            let next_id = logits
                .argmax(D::Minus1)?
                .to_scalar::<u32>()?;
            output_ids.push(next_id);

            if next_id == config.eos_token_id || next_id == config.forced_eos_token_id {
                break;
            }
        }

        // Remove decoder_start_token_id from output
        if output_ids.first() == Some(&config.decoder_start_token_id) {
            output_ids.remove(0);
        }

        // Detokenize
        let decoded = loaded
            .tgt_tokenizer
            .decode(&output_ids, true)
            .map_err(|e| anyhow::anyhow!("Detokenization failed: {}", e))?;

        Ok(decoded)
    }

    /// Hardcoded MarianConfig for opus-mt-es-en (same architecture as en-es, reversed).
    fn opus_mt_es_en() -> marian::Config {
        marian::Config {
            vocab_size: 65001,
            decoder_vocab_size: None,
            max_position_embeddings: 512,
            encoder_layers: 6,
            encoder_ffn_dim: 2048,
            encoder_attention_heads: 8,
            decoder_layers: 6,
            decoder_ffn_dim: 2048,
            decoder_attention_heads: 8,
            use_cache: true,
            is_encoder_decoder: true,
            activation_function: candle_nn::Activation::Swish,
            d_model: 512,
            decoder_start_token_id: 65000,
            scale_embedding: true,
            pad_token_id: 65000,
            eos_token_id: 0,
            forced_eos_token_id: 0,
            share_encoder_decoder_embeddings: true,
        }
    }

    /// Build a MarianConfig for ES→EN by loading config.json from HuggingFace.
    fn build_config_from_repo(&self) -> anyhow::Result<marian::Config> {
        let info = MarianModelInfo::es_en();
        let url = format!(
            "https://huggingface.co/{}/resolve/{}/config.json",
            info.model_repo, info.model_revision
        );

        // Download config.json synchronously using reqwest blocking
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;
        let config_str = client.get(&url).send()?.error_for_status()?.text()?;

        let json: serde_json::Value = serde_json::from_str(&config_str)?;

        let activation = match json["activation_function"].as_str().unwrap_or("swish") {
            "relu" => candle_nn::Activation::Relu,
            "swish" => candle_nn::Activation::Swish,
            "gelu" => candle_nn::Activation::Gelu,
            _ => candle_nn::Activation::Swish,
        };

        Ok(marian::Config {
            vocab_size: json["vocab_size"].as_u64().unwrap_or(65001) as usize,
            decoder_vocab_size: json["decoder_vocab_size"].as_u64().map(|v| v as usize),
            max_position_embeddings: json["max_position_embeddings"].as_u64().unwrap_or(512) as usize,
            encoder_layers: json["encoder_layers"].as_u64().unwrap_or(6) as usize,
            encoder_ffn_dim: json["encoder_ffn_dim"].as_u64().unwrap_or(2048) as usize,
            encoder_attention_heads: json["encoder_attention_heads"].as_u64().unwrap_or(8) as usize,
            decoder_layers: json["decoder_layers"].as_u64().unwrap_or(6) as usize,
            decoder_ffn_dim: json["decoder_ffn_dim"].as_u64().unwrap_or(2048) as usize,
            decoder_attention_heads: json["decoder_attention_heads"].as_u64().unwrap_or(8) as usize,
            use_cache: json["use_cache"].as_bool().unwrap_or(true),
            is_encoder_decoder: json["is_encoder_decoder"].as_bool().unwrap_or(true),
            activation_function: activation,
            d_model: json["d_model"].as_u64().unwrap_or(512) as usize,
            decoder_start_token_id: json["decoder_start_token_id"].as_u64().unwrap_or(65000) as u32,
            scale_embedding: json["scale_embedding"].as_bool().unwrap_or(true),
            pad_token_id: json["pad_token_id"].as_u64().unwrap_or(65000) as u32,
            eos_token_id: json["eos_token_id"].as_u64().unwrap_or(0) as u32,
            forced_eos_token_id: json["forced_eos_token_id"].as_u64().unwrap_or(0) as u32,
            share_encoder_decoder_embeddings: json["share_encoder_decoder_embeddings"].as_bool().unwrap_or(true),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_engine() -> MarianEngine {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "marian_engine_test_{}_{:?}",
            std::process::id(),
            id
        ));
        MarianEngine::new(dir)
    }

    #[test]
    fn test_supported_pairs() {
        assert!(MarianEngine::is_supported("en", "es"));
        assert!(MarianEngine::is_supported("es", "en"));
        assert!(!MarianEngine::is_supported("en", "fr"));
        assert!(!MarianEngine::is_supported("es", "fr"));
        assert!(!MarianEngine::is_supported("auto", "es"));
    }

    #[test]
    fn test_not_loaded_initially() {
        let engine = temp_engine();
        assert!(!engine.is_downloaded("en", "es"));
    }

    #[test]
    fn test_translate_without_load_fails() {
        let mut engine = temp_engine();
        let result = engine.translate("hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_direction_pair_key() {
        assert_eq!(TranslationDirection::EnToEs.pair_key(), "en-es");
        assert_eq!(TranslationDirection::EsToEn.pair_key(), "es-en");
    }

    #[test]
    fn test_direction_from_langs() {
        assert_eq!(TranslationDirection::from_langs("en", "es"), Some(TranslationDirection::EnToEs));
        assert_eq!(TranslationDirection::from_langs("es", "en"), Some(TranslationDirection::EsToEn));
        assert_eq!(TranslationDirection::from_langs("en", "fr"), None);
    }
}
