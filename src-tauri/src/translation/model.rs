use std::path::PathBuf;
use tracing::info;

/// Metadata for a Marian translation model.
#[derive(Debug, Clone)]
pub struct MarianModelInfo {
    /// Language pair key (e.g., "en-es", "es-en")
    pub pair: String,
    /// HuggingFace repo for model weights (owner/name)
    pub model_repo: String,
    /// Branch/PR containing safetensors
    pub model_revision: String,
    /// HuggingFace repo for tokenizers
    pub tokenizer_repo: String,
    /// Source tokenizer filename in tokenizer_repo
    pub src_tokenizer_filename: String,
    /// Target tokenizer filename in tokenizer_repo
    pub tgt_tokenizer_filename: String,
}

impl MarianModelInfo {
    pub fn en_es() -> Self {
        Self {
            pair: "en-es".to_string(),
            model_repo: "Helsinki-NLP/opus-mt-en-es".to_string(),
            model_revision: "refs/pr/4".to_string(),
            tokenizer_repo: "KeighBee/candle-marian".to_string(),
            src_tokenizer_filename: "tokenizer-marian-base-en-es-en.json".to_string(),
            tgt_tokenizer_filename: "tokenizer-marian-base-en-es-es.json".to_string(),
        }
    }

    pub fn es_en() -> Self {
        Self {
            pair: "es-en".to_string(),
            model_repo: "Helsinki-NLP/opus-mt-es-en".to_string(),
            model_revision: "refs/pr/6".to_string(),
            tokenizer_repo: "KeighBee/candle-marian".to_string(),
            src_tokenizer_filename: "tokenizer-marian-base-en-es-es.json".to_string(),
            tgt_tokenizer_filename: "tokenizer-marian-base-en-es-en.json".to_string(),
        }
    }

    pub fn supported_pairs() -> Vec<&'static str> {
        vec!["en-es", "es-en"]
    }

    fn parse_repo(repo_str: &str) -> (&str, &str) {
        let mut parts = repo_str.splitn(2, '/');
        let owner = parts.next().unwrap_or("");
        let name = parts.next().unwrap_or("");
        (owner, name)
    }
}

pub struct MarianModelManager {
    models_dir: PathBuf,
}

impl MarianModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    /// Get the models directory path.
    pub fn models_dir(&self) -> PathBuf {
        self.models_dir.clone()
    }

    /// Returns the directory for a specific language pair's files.
    fn pair_dir(&self, pair: &str) -> PathBuf {
        self.models_dir.join("marian").join(pair)
    }

    /// Check if the model (safetensors + tokenizers) is downloaded for a pair.
    pub fn is_downloaded(&self, pair: &str) -> bool {
        let dir = self.pair_dir(pair);
        dir.join("model.safetensors").exists()
            && dir.join("src_tokenizer.json").exists()
            && dir.join("tgt_tokenizer.json").exists()
    }

    /// Get the path to the model.safetensors file for a pair.
    pub fn model_path(&self, pair: &str) -> PathBuf {
        self.pair_dir(pair).join("model.safetensors")
    }

    /// Get the path to the source tokenizer for a pair.
    pub fn src_tokenizer_path(&self, pair: &str) -> PathBuf {
        self.pair_dir(pair).join("src_tokenizer.json")
    }

    /// Get the path to the target tokenizer for a pair.
    pub fn tgt_tokenizer_path(&self, pair: &str) -> PathBuf {
        self.pair_dir(pair).join("tgt_tokenizer.json")
    }

    /// Download model files for a language pair using hf-hub 1.x (native Rust, xet-backed).
    pub async fn download_async(&self, info: &MarianModelInfo) -> anyhow::Result<()> {
        let pair_dir = self.pair_dir(&info.pair);
        std::fs::create_dir_all(&pair_dir)?;

        info!("Downloading Marian model for {}...", info.pair);

        let client = hf_hub::HFClientSync::new()
            .map_err(|e| anyhow::anyhow!("Failed to create HF client: {}", e))?;

        // 1. Download model.safetensors from the model repo (may be xet-backed)
        let model_path = pair_dir.join("model.safetensors");
        if !model_path.exists() {
            let (owner, name) = MarianModelInfo::parse_repo(&info.model_repo);
            info!("  Downloading model.safetensors from {}...", info.model_repo);
            let repo = client.model(owner, name);
            repo.download_file()
                .filename("model.safetensors")
                .revision(info.model_revision.clone())
                .local_dir(pair_dir.clone())
                .send()
                .map_err(|e| anyhow::anyhow!("Failed to download model.safetensors: {}", e))?;
            info!("  -> {}", model_path.display());
        } else {
            info!("  model.safetensors already exists for {}", info.pair);
        }

        // 2. Download source tokenizer from KeighBee/candle-marian
        let src_path = pair_dir.join("src_tokenizer.json");
        if !src_path.exists() {
            let (owner, name) = MarianModelInfo::parse_repo(&info.tokenizer_repo);
            info!("  Downloading source tokenizer...");
            let repo = client.model(owner, name);
            let downloaded = repo
                .download_file()
                .filename(info.src_tokenizer_filename.clone())
                .local_dir(pair_dir.clone())
                .send()
                .map_err(|e| anyhow::anyhow!("Failed to download source tokenizer: {}", e))?;

            // Rename to standard name if needed
            if downloaded != src_path {
                std::fs::rename(&downloaded, &src_path)?;
            }
            info!("  -> {}", src_path.display());
        } else {
            info!("  src_tokenizer.json already exists for {}", info.pair);
        }

        // 3. Download target tokenizer from KeighBee/candle-marian
        let tgt_path = pair_dir.join("tgt_tokenizer.json");
        if !tgt_path.exists() {
            let (owner, name) = MarianModelInfo::parse_repo(&info.tokenizer_repo);
            info!("  Downloading target tokenizer...");
            let repo = client.model(owner, name);
            let downloaded = repo
                .download_file()
                .filename(info.tgt_tokenizer_filename.clone())
                .local_dir(pair_dir.clone())
                .send()
                .map_err(|e| anyhow::anyhow!("Failed to download target tokenizer: {}", e))?;

            // Rename to standard name if needed
            if downloaded != tgt_path {
                std::fs::rename(&downloaded, &tgt_path)?;
            }
            info!("  -> {}", tgt_path.display());
        } else {
            info!("  tgt_tokenizer.json already exists for {}", info.pair);
        }

        // Verify all files exist
        if !self.is_downloaded(&info.pair) {
            return Err(anyhow::anyhow!(
                "Download completed but files missing for {}",
                info.pair
            ));
        }

        let size_mb = self.size_bytes(&info.pair) as f64 / (1024.0 * 1024.0);
        info!("Model {} ready ({:.1} MB)", info.pair, size_mb);
        Ok(())
    }

    /// Delete model files for a language pair.
    pub fn delete(&self, pair: &str) -> anyhow::Result<()> {
        let dir = self.pair_dir(pair);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
            info!("Deleted Marian model files for {}", pair);
        }
        Ok(())
    }

    /// Get total size of model files for a pair in bytes.
    pub fn size_bytes(&self, pair: &str) -> u64 {
        let dir = self.pair_dir(pair);
        let mut total = 0;
        for name in &["model.safetensors", "src_tokenizer.json", "tgt_tokenizer.json"] {
            let path = dir.join(name);
            if let Ok(meta) = std::fs::metadata(&path) {
                total += meta.len();
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_manager() -> MarianModelManager {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "marian_model_test_{}_{:?}",
            std::process::id(),
            id
        ));
        MarianModelManager::new(dir)
    }

    #[test]
    fn test_supported_pairs() {
        let pairs = MarianModelInfo::supported_pairs();
        assert!(pairs.contains(&"en-es"));
        assert!(pairs.contains(&"es-en"));
    }

    #[test]
    fn test_not_downloaded_initially() {
        let mgr = temp_manager();
        assert!(!mgr.is_downloaded("en-es"));
        assert!(!mgr.is_downloaded("es-en"));
    }

    #[test]
    fn test_model_info_fields() {
        let info = MarianModelInfo::en_es();
        assert_eq!(info.pair, "en-es");
        assert_eq!(info.model_repo, "Helsinki-NLP/opus-mt-en-es");
        assert_eq!(info.model_revision, "refs/pr/4");
        assert_eq!(info.tokenizer_repo, "KeighBee/candle-marian");

        let info = MarianModelInfo::es_en();
        assert_eq!(info.pair, "es-en");
        assert_eq!(info.model_repo, "Helsinki-NLP/opus-mt-es-en");
        assert_eq!(info.model_revision, "refs/pr/6");
    }

    #[test]
    fn test_delete_nonexistent_is_ok() {
        let mgr = temp_manager();
        assert!(mgr.delete("en-es").is_ok());
    }

    #[test]
    fn test_size_bytes_empty() {
        let mgr = temp_manager();
        assert_eq!(mgr.size_bytes("en-es"), 0);
    }
}
