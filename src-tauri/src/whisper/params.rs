use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionParams {
    pub language: Option<String>,
    pub threads: u32,
    pub gpu: bool,
    pub translate: bool,
    #[serde(default = "default_beam_size")]
    pub beam_size: usize,
    #[serde(default = "default_compute_type")]
    pub compute_type: String,
}

fn default_beam_size() -> usize {
    5
}

fn default_compute_type() -> String {
    "float16".to_string()
}

impl Default for TranscriptionParams {
    fn default() -> Self {
        Self {
            language: Some("auto".to_string()),
            threads: 4,
            gpu: false,
            translate: false,
            beam_size: default_beam_size(),
            compute_type: default_compute_type(),
        }
    }
}
