use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiProviderType {
    #[serde(alias = "ollama")]
    Ollama,
    #[serde(alias = "lmstudio", alias = "lm_studio")]
    LmStudio,
    #[serde(alias = "deepseek")]
    DeepSeek,
}

impl AiProviderType {
    pub fn display_name(&self) -> &str {
        match self {
            AiProviderType::Ollama => "Ollama",
            AiProviderType::LmStudio => "LM Studio",
            AiProviderType::DeepSeek => "DeepSeek",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Ollama, Self::LmStudio, Self::DeepSeek]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiConfig {
    pub provider: AiProviderType,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

impl AiConfig {
    pub fn default_for(provider: &AiProviderType) -> Self {
        match provider {
            AiProviderType::Ollama => Self {
                provider: AiProviderType::Ollama,
                base_url: "http://localhost:11434/v1".to_string(),
                api_key: Some("ollama".to_string()),
                model: "llama3.2".to_string(),
            },
            AiProviderType::LmStudio => Self {
                provider: AiProviderType::LmStudio,
                base_url: "http://localhost:1234/v1".to_string(),
                api_key: Some("not-needed".to_string()),
                model: "local-model".to_string(),
            },
            AiProviderType::DeepSeek => Self {
                provider: AiProviderType::DeepSeek,
                base_url: "https://api.deepseek.com/v1".to_string(),
                api_key: None,
                model: "deepseek-chat".to_string(),
            },
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self::default_for(&AiProviderType::Ollama)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        Self {
            role: ChatRole::System,
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: ChatRole::User,
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderInfo {
    pub provider_type: AiProviderType,
    pub display_name: String,
    pub default_url: String,
    pub needs_api_key: bool,
}

impl AiProviderInfo {
    pub fn list() -> Vec<Self> {
        AiProviderType::all()
            .iter()
            .map(|p| {
                let config = AiConfig::default_for(p);
                Self {
                    provider_type: p.clone(),
                    display_name: p.display_name().to_string(),
                    default_url: config.base_url,
                    needs_api_key: matches!(p, AiProviderType::DeepSeek),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_defaults() {
        let ollama = AiConfig::default_for(&AiProviderType::Ollama);
        assert_eq!(ollama.base_url, "http://localhost:11434/v1");
        assert_eq!(ollama.model, "llama3.2");

        let lmstudio = AiConfig::default_for(&AiProviderType::LmStudio);
        assert_eq!(lmstudio.base_url, "http://localhost:1234/v1");

        let deepseek = AiConfig::default_for(&AiProviderType::DeepSeek);
        assert!(deepseek.api_key.is_none());
    }

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful");
        assert_eq!(sys.role, ChatRole::System);

        let usr = ChatMessage::user("Hello");
        assert_eq!(usr.role, ChatRole::User);

        let asst = ChatMessage::assistant("Hi there");
        assert_eq!(asst.role, ChatRole::Assistant);
    }

    #[test]
    fn test_provider_info_list() {
        let providers = AiProviderInfo::list();
        assert_eq!(providers.len(), 3);
        assert!(providers.iter().any(|p| p.provider_type == AiProviderType::Ollama));
        assert!(providers.iter().any(|p| p.provider_type == AiProviderType::DeepSeek));
    }
}
