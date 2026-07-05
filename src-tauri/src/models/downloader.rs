use std::path::PathBuf;
use reqwest;
use sha2::{Sha256, Digest};
use tracing::info;

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub filename: String,
    pub url: String,
    pub size_mb: u64,
    pub sha256: String,
}

impl ModelInfo {
    pub fn available_models() -> Vec<Self> {
        vec![
            Self {
                name: "tiny".to_string(),
                filename: "ggml-tiny.bin".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".to_string(),
                size_mb: 39,
                sha256: String::new(),
            },
            Self {
                name: "base".to_string(),
                filename: "ggml-base.bin".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string(),
                size_mb: 142,
                sha256: String::new(),
            },
            Self {
                name: "small".to_string(),
                filename: "ggml-small.bin".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string(),
                size_mb: 466,
                sha256: String::new(),
            },
            Self {
                name: "medium".to_string(),
                filename: "ggml-medium.bin".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
                size_mb: 1500,
                sha256: String::new(),
            },
            Self {
                name: "large-v3".to_string(),
                filename: "ggml-large-v3.bin".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".to_string(),
                size_mb: 3100,
                sha256: String::new(),
            },
        ]
    }
}

pub struct ModelDownloader {
    models_dir: PathBuf,
}

impl ModelDownloader {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    pub async fn download(&self, model_name: &str) -> anyhow::Result<PathBuf> {
        let models = ModelInfo::available_models();
        let model = models
            .iter()
            .find(|m| m.name == model_name)
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))?;

        let dest_path = self.models_dir.join(&model.filename);
        if dest_path.exists() {
            info!("Model '{}' already downloaded", model_name);
            return Ok(dest_path);
        }

        std::fs::create_dir_all(&self.models_dir)?;

        info!("Downloading model '{}' ({} MB)...", model_name, model.size_mb);

        let client = reqwest::Client::new();
        let response = client.get(&model.url).send().await?;
        let bytes = response.bytes().await?;

        std::fs::write(&dest_path, &bytes)?;

        info!("Model '{}' downloaded successfully", model_name);
        Ok(dest_path)
    }

    pub fn verify_checksum(&self, model_name: &str, expected_sha256: &str) -> anyhow::Result<bool> {
        let models = ModelInfo::available_models();
        let model = models
            .iter()
            .find(|m| m.name == model_name)
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))?;

        let path = self.models_dir.join(&model.filename);
        if !path.exists() {
            return Ok(false);
        }

        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        let hex_result = hex::encode(result);

        Ok(hex_result == expected_sha256)
    }
}
