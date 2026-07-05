use std::path::PathBuf;
use anyhow::Result;
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

pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    pub fn ensure_models_dir(&self) -> Result<()> {
        if !self.models_dir.exists() {
            std::fs::create_dir_all(&self.models_dir)?;
        }
        Ok(())
    }

    pub fn list_downloaded(&self) -> Vec<String> {
        let mut models = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("ggml-") && name.ends_with(".bin") {
                        models.push(name.to_string());
                    }
                }
            }
        }
        models
    }

    pub fn is_downloaded(&self, model_name: &str) -> bool {
        let filename = format!("ggml-{}.bin", model_name);
        self.models_dir.join(filename).exists()
    }

    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(format!("ggml-{}.bin", model_name))
    }

    pub fn models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    pub async fn download_model(&self, model_name: &str) -> Result<PathBuf> {
        let models = ModelInfo::available_models();
        let model = models
            .iter()
            .find(|m| m.name == model_name)
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))?;

        let dest_path = self.get_model_path(model_name);
        if dest_path.exists() {
            info!("Model '{}' already downloaded", model_name);
            return Ok(dest_path);
        }

        info!("Downloading model '{}' ({} MB)...", model_name, model.size_mb);

        let client = reqwest::Client::new();
        let response = client.get(&model.url).send().await?;
        let bytes = response.bytes().await?;

        std::fs::write(&dest_path, &bytes)?;

        info!("Model '{}' downloaded successfully", model_name);
        Ok(dest_path)
    }

    pub fn delete_model(&self, model_name: &str) -> Result<()> {
        let path = self.get_model_path(model_name);
        if path.exists() {
            std::fs::remove_file(path)?;
            info!("Model '{}' deleted", model_name);
        }
        Ok(())
    }

    pub fn verify_checksum(&self, model_name: &str, expected_sha256: &str) -> Result<bool> {
        let path = self.get_model_path(model_name);
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
