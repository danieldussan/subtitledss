use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone)]
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    pub fn ensure_models_dir(&self) -> anyhow::Result<()> {
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

    pub fn delete_model(&self, model_name: &str) -> anyhow::Result<()> {
        let path = self.get_model_path(model_name);
        if path.exists() {
            std::fs::remove_file(path)?;
            info!("Model '{}' deleted", model_name);
        }
        Ok(())
    }

    pub fn get_model_size(&self, model_name: &str) -> u64 {
        let path = self.get_model_path(model_name);
        if path.exists() {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        }
    }
}
