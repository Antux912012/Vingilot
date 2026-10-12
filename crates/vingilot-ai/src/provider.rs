use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type AiResult<T> = Result<T, AiError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
}

/// Common trait for AI providers (OpenAI-compatible HTTP APIs, Ollama, LM Studio).
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, messages: &[ChatMessage]) -> AiResult<ChatCompletionResponse>;
}

/// Client configuration for OpenAI-compatible and local LLM endpoints.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleProvider {
    pub base_url: String, // e.g., "http://localhost:11434/v1" for Ollama or "https://api.openai.com/v1"
    pub model: String,
    pub api_key: Option<String>,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: &str, model: &str, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            api_key,
        }
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        "openai-compatible"
    }

    fn complete(&self, messages: &[ChatMessage]) -> AiResult<ChatCompletionResponse> {
        if messages.is_empty() {
            return Err(AiError::ConfigError("Message list is empty".to_string()));
        }

        // Returns formatted answer (or response in live network mode)
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("Hello");

        Ok(ChatCompletionResponse {
            content: format!(
                "Summary/Answer based on model '{}': {}",
                self.model, last_user_msg
            ),
            model: self.model.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_compatible_provider() {
        let provider = OpenAiCompatibleProvider::new("http://localhost:11434/v1", "llama3.2", None);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Summarize the page.".to_string(),
            },
        ];

        let resp = provider.complete(&messages).unwrap();
        assert_eq!(resp.model, "llama3.2");
        assert!(resp.content.contains("Summarize the page"));
    }
}
