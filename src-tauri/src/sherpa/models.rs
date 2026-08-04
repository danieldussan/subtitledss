use std::path::{Path, PathBuf};

use reqwest;
use tracing::info;

use super::engine::SherpaModelKind;

#[derive(Debug, Clone)]
pub struct SherpaModelFile {
    pub filename: &'static str,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct SherpaModelInfo {
    pub name: &'static str,
    pub label: &'static str,
    pub kind: SherpaModelKind,
    pub languages: &'static str,
    pub description: &'static str,
    pub size_mb: u64,
    pub files: Vec<SherpaModelFile>,
}

impl SherpaModelInfo {
    pub fn available() -> Vec<Self> {
        let hf = "https://huggingface.co";
        vec![
            Self {
                name: "sense-voice-int8",
                label: "SenseVoice Small",
                kind: SherpaModelKind::SenseVoice,
                languages: "Chinese, English, Japanese, Korean, Cantonese",
                description: "Ultra-fast ASR. No Spanish support.",
                size_mb: 239,
                files: files_from_repo(
                    &format!(
                        "{hf}/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/resolve/main"
                    ),
                    &["model.int8.onnx", "tokens.txt"],
                ),
            },
            Self {
                name: "parakeet-tdt-0.6b-v3-int8",
                label: "Parakeet TDT 0.6B v3",
                kind: SherpaModelKind::ParakeetTdt,
                languages: "25 European languages (incl. Spanish)",
                description: "Fastest + most accurate for Spanish. Recommended for real-time.",
                size_mb: 670,
                files: files_from_repo(
                    &format!(
                        "{hf}/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/main"
                    ),
                    &[
                        "encoder.int8.onnx",
                        "decoder.int8.onnx",
                        "joiner.int8.onnx",
                        "tokens.txt",
                    ],
                ),
            },
            Self {
                name: "canary-1b-v2-int8",
                label: "Canary 1B v2",
                kind: SherpaModelKind::Canary,
                languages: "25 European languages (ASR/translation)",
                description: "High-accuracy ASR. Slower than Parakeet.",
                size_mb: 1416,
                files: files_from_repo(
                    &format!(
                        "{hf}/Sarphix/canary-1b-v2-sherpa-onnx-int8/resolve/main"
                    ),
                    &["encoder.int8.onnx", "decoder.int8.onnx", "tokens.txt"],
                ),
            },
        ]
    }

    pub fn find(name: &str) -> Option<Self> {
        Self::available().into_iter().find(|m| m.name == name)
    }

    /// `models_dir/sherpa/<name>/` — where the package is downloaded/extracted.
    pub fn model_dir(models_dir: &Path, name: &str) -> PathBuf {
        models_dir.join("sherpa").join(name)
    }
}

pub fn is_sherpa_model(name: &str) -> bool {
    SherpaModelInfo::find(name).is_some()
}

pub fn model_dir(models_dir: &Path, name: &str) -> PathBuf {
    SherpaModelInfo::model_dir(models_dir, name)
}

pub fn list_downloaded(models_dir: &Path) -> Vec<String> {
    let sherpa_dir = models_dir.join("sherpa");
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&sherpa_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

pub async fn download(model_name: &str, models_dir: &Path) -> anyhow::Result<PathBuf> {
    let info = SherpaModelInfo::find(model_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown sherpa model '{}'", model_name))?;

    let dir = SherpaModelInfo::model_dir(models_dir, model_name);
    std::fs::create_dir_all(&dir)?;

    let client = reqwest::Client::new();
    for f in &info.files {
        let dest = dir.join(f.filename);
        if dest.exists() {
            info!("{} already downloaded", f.filename);
            continue;
        }

        info!("Downloading {} from {}", f.filename, f.url);
        let response = client.get(&f.url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download {} (HTTP {})",
                f.filename,
                response.status()
            ));
        }
        let bytes = response.bytes().await?;
        std::fs::write(&dest, &bytes)?;
    }

    info!("Sherpa model '{}' downloaded to {:?}", model_name, dir);
    Ok(dir)
}

fn files_from_repo(base: &str, names: &[&'static str]) -> Vec<SherpaModelFile> {
    names
        .iter()
        .map(|n| SherpaModelFile {
            filename: n,
            url: format!("{base}/{n}"),
        })
        .collect()
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
            "subtitledss_sherpa_test_{}_{:?}",
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn catalog_has_three_models() {
        let models = SherpaModelInfo::available();
        assert_eq!(models.len(), 3);
        assert!(SherpaModelInfo::find("sense-voice-int8").is_some());
        assert!(SherpaModelInfo::find("parakeet-tdt-0.6b-v3-int8").is_some());
        assert!(SherpaModelInfo::find("canary-1b-v2-int8").is_some());
        assert!(SherpaModelInfo::find("nope").is_none());
    }

    #[test]
    fn catalog_files_have_hf_urls() {
        for m in SherpaModelInfo::available() {
            assert!(!m.files.is_empty());
            assert!(m.size_mb > 0);
            assert!(!m.description.is_empty());
            for f in &m.files {
                assert!(f.url.starts_with("https://huggingface.co/"));
                assert!(f.url.ends_with(&f.filename.to_string()));
            }
        }
    }

    #[test]
    fn kind_inference_from_dir_name() {
        assert_eq!(
            SherpaModelKind::from_dir_name("sense-voice-zh-en-ja-ko-yue-int8"),
            Some(SherpaModelKind::SenseVoice)
        );
        assert_eq!(
            SherpaModelKind::from_dir_name("sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8"),
            Some(SherpaModelKind::ParakeetTdt)
        );
        assert_eq!(
            SherpaModelKind::from_dir_name("canary-1b-v2-int8"),
            Some(SherpaModelKind::Canary)
        );
        assert_eq!(SherpaModelKind::from_dir_name("ggml-base.bin"), None);
    }

    #[test]
    fn model_dir_layout() {
        let dir = temp_dir();
        let path = SherpaModelInfo::model_dir(&dir, "canary-1b-v2-int8");
        assert_eq!(
            path,
            dir.join("sherpa").join("canary-1b-v2-int8")
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn list_downloaded_only_dirs() {
        let dir = temp_dir();
        let sherpa_dir = dir.join("sherpa");
        fs::create_dir_all(sherpa_dir.join("canary-1b-v2-int8")).unwrap();
        fs::write(sherpa_dir.join("stray.bin"), b"x").unwrap();

        let mut names = list_downloaded(&dir);
        names.sort();
        assert_eq!(names, vec!["canary-1b-v2-int8"]);

        fs::remove_dir_all(&dir).ok();
    }
}
