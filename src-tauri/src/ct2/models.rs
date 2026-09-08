use std::path::{Path, PathBuf};

use reqwest;
use tracing::info;

/// Files every ct2 model package needs for `ct2rs::Whisper::new` plus the
/// metadata usually shipped with Systran's faster-whisper repos.
pub const CT2_MODEL_FILES: &[&str] = &[
    "model.bin",
    "config.json",
    "tokenizer.json",
    "preprocessor_config.json",
    "vocabulary.json",
    "generation_config.json",
];

#[derive(Debug, Clone)]
pub struct Ct2ModelInfo {
    pub name: &'static str,
    pub label: &'static str,
    pub size_mb: u64,
    pub description: &'static str,
}

impl Ct2ModelInfo {
    /// Systran/faster-whisper-* repos, pre-converted to CTranslate2 format.
    ///
    /// Names are prefixed (`faster-whisper-base`) so they don't collide with
    /// the ggml whisper.cpp catalog (`base`, `tiny`, ...).
    pub fn available() -> Vec<Self> {
        vec![
            Self {
                name: "faster-whisper-tiny",
                label: "faster-whisper tiny",
                size_mb: 75,
                description: "Smallest & fastest. Lower accuracy, multilingual.",
            },
            Self {
                name: "faster-whisper-base",
                label: "faster-whisper base",
                size_mb: 145,
                description: "Small, fast, decent accuracy. Multilingual.",
            },
            Self {
                name: "faster-whisper-small",
                label: "faster-whisper small",
                size_mb: 466,
                description: "Good accuracy/speed balance. Multilingual.",
            },
            Self {
                name: "faster-whisper-medium",
                label: "faster-whisper medium",
                size_mb: 1540,
                description: "High accuracy. Multilingual, slower.",
            },
            Self {
                name: "faster-whisper-large-v3",
                label: "faster-whisper large-v3",
                size_mb: 3090,
                description: "Best accuracy. Heavy, requires GPU for real-time.",
            },
            Self {
                name: "faster-whisper-turbo",
                label: "faster-whisper turbo",
                size_mb: 1620,
                description: "Large-v3 quality with 8x2 decode. Spanish strong.",
            },
        ]
    }

    /// Repository suffix on HF (e.g. `faster-whisper-base` → `base`).
    pub fn repo_name(&self) -> &str {
        self.name
            .strip_prefix("faster-whisper-")
            .unwrap_or(self.name)
    }

    pub fn find(name: &str) -> Option<Self> {
        Self::available().into_iter().find(|m| m.name == name)
    }

    /// `models_dir/ct2/<name>/` — the directory Whisper::new expects.
    pub fn model_dir(models_dir: &Path, name: &str) -> PathBuf {
        models_dir.join("ct2").join(name)
    }
}

pub fn is_ct2_model(name: &str) -> bool {
    Ct2ModelInfo::find(name).is_some()
}

pub fn model_dir(models_dir: &Path, name: &str) -> PathBuf {
    Ct2ModelInfo::model_dir(models_dir, name)
}

/// True when every required file exists (and model.bin is non-empty).
pub fn is_complete(dir: &Path) -> bool {
    CT2_MODEL_FILES.iter().all(|f| {
        let p = dir.join(f);
        p.is_file() && (f != &"model.bin" || p.metadata().map(|m| m.len() > 0).unwrap_or(false))
    })
}

pub fn list_downloaded(models_dir: &Path) -> Vec<String> {
    let ct2_dir = models_dir.join("ct2");
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ct2_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() && is_complete(&entry.path()) {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

pub async fn download(model_name: &str, models_dir: &Path) -> anyhow::Result<PathBuf> {
    let info = Ct2ModelInfo::find(model_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown CTranslate2 model '{}'", model_name))?;

    let dir = Ct2ModelInfo::model_dir(models_dir, model_name);
    std::fs::create_dir_all(&dir)?;

    let client = reqwest::Client::new();
    let base = format!(
        "https://huggingface.co/Systran/faster-whisper-{}/resolve/main",
        info.repo_name()
    );

    for filename in CT2_MODEL_FILES {
        let dest = dir.join(filename);
        if dest.is_file() && dest.metadata().map(|m| m.len() > 0).unwrap_or(false) {
            info!("{} already downloaded", filename);
            continue;
        }

        let url = format!("{}/{}", base, filename);
        info!("Downloading {} from {}", filename, url);
        let response = client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download {} (HTTP {})",
                filename,
                response.status()
            ));
        }
        let bytes = response.bytes().await?;
        std::fs::write(&dest, &bytes)?;
    }

    info!("CTranslate2 model '{}' downloaded to {:?}", model_name, dir);
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "subtitledss_ct2_test_{}_{:?}",
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn catalog_has_six_models() {
        let models = Ct2ModelInfo::available();
        assert_eq!(models.len(), 6);
        assert!(Ct2ModelInfo::find("faster-whisper-turbo").is_some());
        assert!(Ct2ModelInfo::find("faster-whisper-large-v3").is_some());
        assert!(Ct2ModelInfo::find("nope").is_none());
        assert_eq!(
            Ct2ModelInfo::find("faster-whisper-base")
                .unwrap()
                .repo_name(),
            "base"
        );
    }

    #[test]
    fn model_dir_layout() {
        let dir = temp_dir();
        let path = Ct2ModelInfo::model_dir(&dir, "faster-whisper-turbo");
        assert_eq!(path, dir.join("ct2").join("faster-whisper-turbo"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn completeness_check() {
        let dir = temp_dir();
        assert!(!is_complete(&dir));

        let mdir = Ct2ModelInfo::model_dir(&dir, "faster-whisper-turbo");
        fs::create_dir_all(&mdir).unwrap();
        for f in CT2_MODEL_FILES {
            let content: &[u8] = if *f == "model.bin" { b"model" } else { b"x" };
            fs::write(mdir.join(f), content).unwrap();
        }
        assert!(is_complete(&mdir));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn incomplete_dir_not_listed() {
        let dir = temp_dir();
        let mdir = Ct2ModelInfo::model_dir(&dir, "faster-whisper-turbo");
        fs::create_dir_all(&mdir).unwrap();
        fs::write(mdir.join("model.bin"), b"m").unwrap();
        assert!(!is_complete(&mdir));
        assert!(list_downloaded(&dir).is_empty());
        fs::remove_dir_all(&dir).ok();
    }
}
