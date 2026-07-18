use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::config::{AiConfig, ChatMessage, ChatRole};

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: Option<OpenAiStreamDelta>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Send a chat completion request and return the full response.
    async fn chat(&self, system_prompt: &str, messages: &[ChatMessage]) -> anyhow::Result<String>;

    /// Send a streaming chat completion request. Returns a receiver that yields tokens.
    async fn chat_stream(
        &self,
        system_prompt: &str,
        messages: &[ChatMessage],
    ) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>>;

    /// Provider display name.
    fn name(&self) -> &str;

    /// Check if the provider is reachable.
    async fn is_available(&self) -> bool;
}

pub struct OpenAiCompatibleProvider {
    config: AiConfig,
    client: Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(config: AiConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    fn build_request(
        &self,
        system_prompt: &str,
        messages: &[ChatMessage],
        stream: bool,
    ) -> OpenAiRequest {
        let mut api_messages = vec![OpenAiMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        }];

        for msg in messages {
            let role = match msg.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Assistant => "assistant",
            };
            api_messages.push(OpenAiMessage {
                role: role.to_string(),
                content: msg.content.clone(),
            });
        }

        OpenAiRequest {
            model: self.config.model.clone(),
            messages: api_messages,
            stream,
            temperature: Some(0.7),
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'))
    }

    fn add_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref api_key) = self.config.api_key {
            builder.bearer_auth(api_key)
        } else {
            builder
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiCompatibleProvider {
    async fn chat(&self, system_prompt: &str, messages: &[ChatMessage]) -> anyhow::Result<String> {
        let request = self.build_request(system_prompt, messages, false);
        let url = self.chat_url();

        info!("AI chat request to {} (model: {})", url, self.config.model);

        let builder = self.client.post(&url).json(&request);
        let response = self.add_auth(builder).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("AI API error {}: {}", status, body));
        }

        let OpenAiResponse { choices } = response.json().await?;
        let content = choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }

    async fn chat_stream(
        &self,
        system_prompt: &str,
        messages: &[ChatMessage],
    ) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>> {
        let request = self.build_request(system_prompt, messages, true);
        let url = self.chat_url();

        info!(
            "AI stream request to {} (model: {})",
            url, self.config.model
        );

        let builder = self.client.post(&url).json(&request);
        let response = self.add_auth(builder).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("AI API error {}: {}", status, body));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            use futures_util::StreamExt;

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let text = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&text);

                        // Process complete SSE lines
                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer = buffer[line_end + 1..].to_string();

                            if line.is_empty() || line.starts_with(':') {
                                continue;
                            }

                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    return;
                                }

                                match serde_json::from_str::<OpenAiStreamChunk>(data) {
                                    Ok(chunk) => {
                                        for choice in &chunk.choices {
                                            if let Some(ref delta) = choice.delta {
                                                if let Some(ref content) = delta.content {
                                                    if tx.send(content.clone()).await.is_err() {
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse stream chunk: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    fn name(&self) -> &str {
        self.config.provider.display_name()
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/models", self.config.base_url.trim_end_matches('/'));
        let builder = self.client.get(&url);
        self.add_auth(builder)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

/// Create an AI provider from configuration.
pub fn create_provider(config: AiConfig) -> Box<dyn AiProvider> {
    Box::new(OpenAiCompatibleProvider::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::config::AiProviderType;

    #[test]
    fn test_build_request_system_message() {
        let config = AiConfig::default_for(&AiProviderType::Ollama);
        let provider = OpenAiCompatibleProvider::new(config);
        let request = provider.build_request("Be helpful", &[], false);

        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[0].content, "Be helpful");
        assert!(!request.stream);
    }

    #[test]
    fn test_build_request_with_messages() {
        let config = AiConfig::default_for(&AiProviderType::Ollama);
        let provider = OpenAiCompatibleProvider::new(config);
        let messages = vec![
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there"),
            ChatMessage::user("How are you?"),
        ];
        let request = provider.build_request("System", &messages, true);

        assert_eq!(request.messages.len(), 4); // system + 3 messages
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[2].role, "assistant");
        assert!(request.stream);
    }

    #[test]
    fn test_chat_url_trims_slash() {
        let config = AiConfig {
            base_url: "http://localhost:11434/v1/".to_string(),
            ..AiConfig::default_for(&AiProviderType::Ollama)
        };
        let provider = OpenAiCompatibleProvider::new(config);
        assert_eq!(
            provider.chat_url(),
            "http://localhost:11434/v1/chat/completions"
        );
    }

    #[test]
    fn test_provider_name() {
        let ollama = OpenAiCompatibleProvider::new(AiConfig::default_for(&AiProviderType::Ollama));
        assert_eq!(ollama.name(), "Ollama");

        let lmstudio =
            OpenAiCompatibleProvider::new(AiConfig::default_for(&AiProviderType::LmStudio));
        assert_eq!(lmstudio.name(), "LM Studio");

        let deepseek =
            OpenAiCompatibleProvider::new(AiConfig::default_for(&AiProviderType::DeepSeek));
        assert_eq!(deepseek.name(), "DeepSeek");
    }

    #[test]
    fn test_create_provider_returns_box() {
        let config = AiConfig::default_for(&AiProviderType::Ollama);
        let provider = create_provider(config);
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn test_add_auth_with_key() {
        let config = AiConfig {
            api_key: Some("test-key".to_string()),
            ..AiConfig::default_for(&AiProviderType::DeepSeek)
        };
        let provider = OpenAiCompatibleProvider::new(config);
        let builder = provider.client.get("http://example.com");
        let request = provider.add_auth(builder).build().unwrap();
        assert_eq!(
            request.headers().get("authorization").unwrap(),
            "Bearer test-key"
        );
    }

    #[test]
    fn test_add_auth_without_key() {
        let config = AiConfig {
            api_key: None,
            ..AiConfig::default_for(&AiProviderType::Ollama)
        };
        let provider = OpenAiCompatibleProvider::new(config);
        let builder = provider.client.get("http://example.com");
        let request = provider.add_auth(builder).build().unwrap();
        assert!(request.headers().get("authorization").is_none());
    }

    #[test]
    fn test_build_request_temperature() {
        let config = AiConfig::default_for(&AiProviderType::Ollama);
        let provider = OpenAiCompatibleProvider::new(config);
        let request = provider.build_request("System", &[], false);
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_build_request_model_cloned() {
        let config = AiConfig {
            model: "custom-model".to_string(),
            ..AiConfig::default_for(&AiProviderType::Ollama)
        };
        let provider = OpenAiCompatibleProvider::new(config);
        let request = provider.build_request("System", &[], false);
        assert_eq!(request.model, "custom-model");
    }
}
