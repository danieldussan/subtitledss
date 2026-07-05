use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionParams {
    pub language: Option<String>,
    pub threads: u32,
    pub gpu: bool,
    pub translate: bool,
}

impl Default for TranscriptionParams {
    fn default() -> Self {
        Self {
            language: Some("auto".to_string()),
            threads: 4,
            gpu: false,
            translate: false,
        }
    }
}
