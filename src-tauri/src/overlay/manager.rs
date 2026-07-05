use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub opacity: f64,
    pub always_on_top: bool,
    pub click_through: bool,
    pub font_size: u32,
    pub font_color: String,
    pub background_color: String,
    pub auto_hide: bool,
    pub auto_hide_delay: u64,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            x: 100,
            y: 500,
            width: 600,
            height: 100,
            opacity: 0.9,
            always_on_top: true,
            click_through: false,
            font_size: 24,
            font_color: "#ffffff".to_string(),
            background_color: "#00000080".to_string(),
            auto_hide: true,
            auto_hide_delay: 5000,
        }
    }
}

pub struct OverlayManager {
    config: OverlayConfig,
    visible: bool,
    last_text: String,
}

impl OverlayManager {
    pub fn new(config: OverlayConfig) -> Self {
        Self {
            config,
            visible: true,
            last_text: String::new(),
        }
    }

    pub fn update_text(&mut self, text: &str) {
        self.last_text = text.to_string();
        if text.is_empty() && self.config.auto_hide {
            self.visible = false;
        } else if !text.is_empty() {
            self.visible = true;
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn config(&self) -> &OverlayConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: OverlayConfig) {
        self.config = config;
    }

    pub fn last_text(&self) -> &str {
        &self.last_text
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new(OverlayConfig::default())
    }
}
