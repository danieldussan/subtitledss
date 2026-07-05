use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioDevice {
    pub fn new(name: String, is_default: bool, sample_rate: u32, channels: u16) -> Self {
        Self {
            name,
            is_default,
            sample_rate,
            channels,
        }
    }
}
