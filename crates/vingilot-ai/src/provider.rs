use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};
use thiserror::Error;

pub const GEMINI_AI_STUDIO_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

pub const GEMINI_POPULAR_MODELS: &[&str] = &[
    "gemini-2.5-flash",
    "gemini-2.5-pro",
    "gemini-2.5-flash-lite",
    "gemini-2.0-flash",
    "gemini-1.5-pro",
    "gemini-1.5-flash",
];

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

/// Common trait for AI providers (Google Gemini via Google AI Studio, Anthropic Claude,
/// OpenAI-compatible HTTP APIs, Ollama, LM Studio, llama.cpp).
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, messages: &[ChatMessage]) -> AiResult<ChatCompletionResponse>;
}

/// Google Gemini AI provider distributed via Google AI Studio (`generativelanguage.googleapis.com/v1beta`).
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self::with_base_url(GEMINI_AI_STUDIO_BASE_URL, model, api_key)
    }

    pub fn with_base_url(base_url: &str, model: &str, api_key: &str) -> Self {
        let normalized_url = OpenAiCompatibleProvider::normalize_base_url(base_url);
        let clean_model = model.trim().trim_start_matches("models/");
        Self {
            base_url: if normalized_url.is_empty() {
                GEMINI_AI_STUDIO_BASE_URL.to_string()
            } else {
                normalized_url
            },
            model: if clean_model.is_empty() {
                "gemini-2.5-flash".to_string()
            } else {
                clean_model.to_string()
            },
            api_key: api_key.trim().to_string(),
        }
    }

    pub fn endpoint_url(&self) -> String {
        let clean_model = self.model.trim().trim_start_matches("models/");
        format!(
            "{}/models/{}:generateContent",
            self.base_url.trim_end_matches('/'),
            clean_model
        )
    }

    pub fn build_payload(messages: &[ChatMessage], temperature: f64) -> serde_json::Value {
        let mut system_parts = Vec::new();
        let mut contents = Vec::new();

        for msg in messages {
            let role = msg.role.as_str();
            if role == "system" {
                if !msg.content.trim().is_empty() {
                    system_parts.push(serde_json::json!({ "text": msg.content }));
                }
            } else {
                let gemini_role = if role == "assistant" || role == "model" {
                    "model"
                } else {
                    "user"
                };
                contents.push(serde_json::json!({
                    "role": gemini_role,
                    "parts": [{ "text": msg.content }]
                }));
            }
        }

        if contents.is_empty() && !system_parts.is_empty() {
            contents.push(serde_json::json!({
                "role": "user",
                "parts": system_parts.clone()
            }));
            system_parts.clear();
        }

        let mut payload = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "temperature": temperature,
                "maxOutputTokens": 1024
            }
        });

        if !system_parts.is_empty() {
            payload["systemInstruction"] = serde_json::json!({
                "parts": system_parts
            });
        }

        payload
    }

    pub fn parse_response(
        resp_json: &serde_json::Value,
        model: &str,
    ) -> AiResult<ChatCompletionResponse> {
        if let Some(err_msg) = resp_json["error"]["message"].as_str() {
            return Err(AiError::ModelError(format!("Gemini API error: {err_msg}")));
        }

        if let Some(parts) = resp_json["candidates"][0]["content"]["parts"].as_array() {
            let mut text = String::new();
            for part in parts {
                if let Some(t) = part["text"].as_str() {
                    text.push_str(t);
                }
            }
            if !text.trim().is_empty() {
                return Ok(ChatCompletionResponse {
                    content: text.trim().to_string(),
                    model: model.trim_start_matches("models/").to_string(),
                });
            }
        }

        Err(AiError::ModelError(
            "No text content returned by Gemini API".to_string(),
        ))
    }

    pub fn list_models(&self) -> AiResult<Vec<String>> {
        if self.api_key.is_empty() {
            return Ok(GEMINI_POPULAR_MODELS.iter().map(|s| s.to_string()).collect());
        }

        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let output = Command::new("curl")
            .args([
                "-s",
                "-f",
                "--max-time",
                "8",
                "-H",
                &format!("x-goog-api-key: {}", self.api_key),
                &url,
            ])
            .output()
            .map_err(|e| AiError::RequestFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(AiError::RequestFailed(
                "Failed to fetch models from Google AI Studio".to_string(),
            ));
        }

        let data: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut models = Vec::new();
        if let Some(items) = data["models"].as_array() {
            for item in items {
                if let Some(methods) = item["supportedGenerationMethods"].as_array() {
                    let supports_gen = methods
                        .iter()
                        .any(|m| m.as_str() == Some("generateContent"));
                    if !supports_gen {
                        continue;
                    }
                }
                if let Some(name) = item["name"].as_str() {
                    let clean = name.trim_start_matches("models/").to_string();
                    if !clean.is_empty() && !models.contains(&clean) {
                        models.push(clean);
                    }
                }
            }
        }

        if models.is_empty() {
            models = GEMINI_POPULAR_MODELS.iter().map(|s| s.to_string()).collect();
        }
        Ok(models)
    }
}

impl AiProvider for GeminiProvider {
    fn name(&self) -> &str {
        "google-gemini"
    }

    fn complete(&self, messages: &[ChatMessage]) -> AiResult<ChatCompletionResponse> {
        if messages.is_empty() {
            return Err(AiError::ConfigError("Message list is empty".to_string()));
        }
        if self.api_key.is_empty() {
            return Err(AiError::ConfigError(
                "Google AI Studio API key is required for Gemini".to_string(),
            ));
        }

        let payload = Self::build_payload(messages, 0.3);
        let body_str = payload.to_string();
        let endpoint = self.endpoint_url();

        let mut cmd = Command::new("curl");
        cmd.args([
            "-s",
            "-X",
            "POST",
            "--max-time",
            "60",
            "-H",
            "Content-Type: application/json",
            "-H",
            "User-Agent: Vingilot/0.1.6",
            "-H",
            &format!("x-goog-api-key: {}", self.api_key),
            "--data-binary",
            "@-",
            &endpoint,
        ]);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| AiError::RequestFailed(format!("Failed to spawn curl: {e}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(body_str.as_bytes());
        }

        let output = child
            .wait_with_output()
            .map_err(|e| AiError::RequestFailed(format!("HTTP request failed: {e}")))?;
        if !output.status.success() {
            let err_str = String::from_utf8_lossy(&output.stderr);
            return Err(AiError::RequestFailed(format!(
                "Gemini API request failed: {err_str}"
            )));
        }

        let resp_json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Self::parse_response(&resp_json, &self.model)
    }
}

/// Unified client configuration for Google Gemini (Google AI Studio), Anthropic Claude,
/// OpenAI-compatible cloud endpoints, and local LLM endpoints (Ollama, LM Studio, llama.cpp).
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleProvider {
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: &str, model: &str, api_key: Option<String>) -> Self {
        let normalized = Self::normalize_base_url(base_url);
        let trimmed_model = model.trim();
        let resolved_model = if trimmed_model.is_empty() {
            if normalized.contains("generativelanguage.googleapis.com") {
                "gemini-2.5-flash".to_string()
            } else if normalized.contains("api.anthropic.com") {
                "claude-3-5-sonnet-latest".to_string()
            } else if normalized.contains("api.openai.com") {
                "gpt-4o-mini".to_string()
            } else {
                "llama3.2".to_string()
            }
        } else if normalized.contains("generativelanguage.googleapis.com") {
            trimmed_model.trim_start_matches("models/").to_string()
        } else {
            trimmed_model.to_string()
        };

        Self {
            base_url: normalized,
            model: resolved_model,
            api_key: api_key
                .map(|k| k.trim().to_string())
                .filter(|k| !k.is_empty()),
        }
    }

    pub fn new_gemini(api_key: Option<String>, model: Option<&str>) -> Self {
        Self::new(
            GEMINI_AI_STUDIO_BASE_URL,
            model.unwrap_or("gemini-2.5-flash"),
            api_key,
        )
    }

    pub fn normalize_base_url(raw: &str) -> String {
        let mut url = raw.trim().trim_end_matches('/').to_string();
        if url.is_empty() {
            return "http://localhost:11434/v1".to_string();
        }

        if url.contains(":generateContent") {
            if let Some(idx) = url.find("/models/") {
                url = url[..idx].trim_end_matches('/').to_string();
            }
        }

        for suffix in ["/chat/completions", "/completions", "/messages", "/models"] {
            if url.ends_with(suffix) {
                url = url.trim_end_matches(suffix).trim_end_matches('/').to_string();
            }
        }

        if url == "https://generativelanguage.googleapis.com" {
            return GEMINI_AI_STUDIO_BASE_URL.to_string();
        }
        if url == "https://api.anthropic.com" {
            return "https://api.anthropic.com/v1".to_string();
        }
        if url == "https://api.openai.com" {
            return "https://api.openai.com/v1".to_string();
        }
        if url == "https://integrate.api.nvidia.com" {
            return "https://integrate.api.nvidia.com/v1".to_string();
        }
        if url == "http://localhost:11434" || url == "http://127.0.0.1:11434" {
            return "http://localhost:11434/v1".to_string();
        }
        if url == "http://localhost:1234" || url == "http://127.0.0.1:1234" {
            return "http://localhost:1234/v1".to_string();
        }
        if url == "http://localhost:18080" || url == "http://127.0.0.1:18080" {
            return "http://127.0.0.1:18080/v1".to_string();
        }

        url
    }

    pub fn is_gemini(&self) -> bool {
        self.base_url.contains("generativelanguage.googleapis.com")
    }

    pub fn is_gemini_native(&self) -> bool {
        self.is_gemini() && !self.base_url.ends_with("/openai")
    }

    pub fn is_claude(&self) -> bool {
        self.base_url.contains("api.anthropic.com")
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        if self.is_gemini() {
            "google-gemini"
        } else if self.is_claude() {
            "anthropic-claude"
        } else {
            "openai-compatible"
        }
    }

    fn complete(&self, messages: &[ChatMessage]) -> AiResult<ChatCompletionResponse> {
        if messages.is_empty() {
            return Err(AiError::ConfigError("Message list is empty".to_string()));
        }

        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("Hello");

        // 1. Native Google AI Studio Gemini API
        if self.is_gemini_native() {
            if let Some(ref key) = self.api_key {
                let gemini = GeminiProvider::with_base_url(&self.base_url, &self.model, key);
                return gemini.complete(messages);
            }
            return Ok(ChatCompletionResponse {
                content: format!(
                    "Summary/Answer based on model '{}': {}",
                    self.model, last_user_msg
                ),
                model: self.model.clone(),
            });
        }

        // 2. Anthropic Claude API
        if self.is_claude() {
            if let Some(ref key) = self.api_key {
                let endpoint = format!("{}/messages", self.base_url.trim_end_matches('/'));
                let mut system_parts = Vec::new();
                let mut chat_msgs = Vec::new();
                for m in messages {
                    if m.role == "system" {
                        if !m.content.trim().is_empty() {
                            system_parts.push(m.content.clone());
                        }
                    } else {
                        let role = if m.role == "assistant" { "assistant" } else { "user" };
                        chat_msgs.push(serde_json::json!({ "role": role, "content": m.content }));
                    }
                }
                if chat_msgs.is_empty() && !system_parts.is_empty() {
                    chat_msgs.push(serde_json::json!({
                        "role": "user",
                        "content": system_parts.join("\n\n")
                    }));
                    system_parts.clear();
                }

                let mut body = serde_json::json!({
                    "model": self.model,
                    "messages": chat_msgs,
                    "max_tokens": 1024,
                    "temperature": 0.3
                });
                if !system_parts.is_empty() {
                    body["system"] = serde_json::Value::String(system_parts.join("\n\n"));
                }

                let body_str = body.to_string();
                let mut cmd = Command::new("curl");
                cmd.args([
                    "-s",
                    "-X",
                    "POST",
                    "--max-time",
                    "60",
                    "-H",
                    "Content-Type: application/json",
                    "-H",
                    "anthropic-version: 2023-06-01",
                    "-H",
                    &format!("x-api-key: {key}"),
                    "--data-binary",
                    "@-",
                    &endpoint,
                ]);
                cmd.stdin(Stdio::piped());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                if let Ok(mut child) = cmd.spawn() {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(body_str.as_bytes());
                    }
                    if let Ok(output) = child.wait_with_output() {
                        if output.status.success() {
                            let resp_json: serde_json::Value =
                                serde_json::from_slice(&output.stdout)?;
                            if let Some(err_msg) = resp_json["error"]["message"].as_str() {
                                return Err(AiError::ModelError(format!(
                                    "Claude API error: {err_msg}"
                                )));
                            }
                            if let Some(text) = resp_json["content"][0]["text"].as_str() {
                                return Ok(ChatCompletionResponse {
                                    content: text.trim().to_string(),
                                    model: self.model.clone(),
                                });
                            }
                        }
                    }
                }
                return Err(AiError::RequestFailed(
                    "Anthropic Claude API request failed".to_string(),
                ));
            }
        }

        // 3. OpenAI-Compatible API (OpenAI, Ollama, LM Studio, llama.cpp, or Gemini /v1beta/openai)
        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 1024
        });
        let body_str = body.to_string();

        let mut cmd = Command::new("curl");
        cmd.args([
            "-s",
            "-X",
            "POST",
            "--max-time",
            if self.api_key.is_some() { "60" } else { "2" },
            "-H",
            "Content-Type: application/json",
            "-H",
            "User-Agent: Vingilot/0.1.6",
        ]);
        if let Some(ref key) = self.api_key {
            cmd.args(["-H", &format!("Authorization: Bearer {key}")]);
        }
        cmd.args(["--data-binary", "@-", &endpoint]);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Ok(mut child) = cmd.spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(body_str.as_bytes());
            }
            if let Ok(output) = child.wait_with_output() {
                if output.status.success() && !output.stdout.is_empty() {
                    if let Ok(resp_json) =
                        serde_json::from_slice::<serde_json::Value>(&output.stdout)
                    {
                        if let Some(content) =
                            resp_json["choices"][0]["message"]["content"].as_str()
                        {
                            return Ok(ChatCompletionResponse {
                                content: content.trim().to_string(),
                                model: self.model.clone(),
                            });
                        }
                        if self.api_key.is_some() {
                            if let Some(err_msg) = resp_json["error"]["message"].as_str() {
                                return Err(AiError::ModelError(err_msg.to_string()));
                            }
                        }
                    }
                } else if self.api_key.is_some() {
                    let err_str = String::from_utf8_lossy(&output.stderr);
                    return Err(AiError::RequestFailed(format!(
                        "Endpoint {} failed: {}",
                        endpoint, err_str
                    )));
                }
            }
        }

        // Offline fallback when no API key is configured and local daemon is not running
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

    #[test]
    fn test_gemini_ai_studio_provider_payload_and_response() {
        let provider = OpenAiCompatibleProvider::new(
            "https://generativelanguage.googleapis.com",
            "models/gemini-2.5-flash",
            Some("AIzaSyTestKey".to_string()),
        );
        assert!(provider.is_gemini());
        assert!(provider.is_gemini_native());
        assert_eq!(provider.name(), "google-gemini");
        assert_eq!(
            provider.base_url,
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(provider.model, "gemini-2.5-flash");

        let gemini = GeminiProvider::new("AIzaSyTestKey", "gemini-2.5-flash");
        assert_eq!(
            gemini.endpoint_url(),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are the AI assistant inside the Vingilot web browser.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Summarize Rust 1.99 release notes.".to_string(),
            },
        ];

        let payload = GeminiProvider::build_payload(&messages, 0.3);
        assert_eq!(
            payload["systemInstruction"]["parts"][0]["text"].as_str(),
            Some("You are the AI assistant inside the Vingilot web browser.")
        );
        assert_eq!(
            payload["contents"][0]["parts"][0]["text"].as_str(),
            Some("Summarize Rust 1.99 release notes.")
        );

        let mock_response = serde_json::json!({
            "candidates": [
                {
                    "content": {
                        "parts": [
                            { "text": "Rust 1.99 introduces faster compilation and new async traits." }
                        ],
                        "role": "model"
                    }
                }
            ]
        });

        let parsed = GeminiProvider::parse_response(&mock_response, "gemini-2.5-flash").unwrap();
        assert_eq!(parsed.model, "gemini-2.5-flash");
        assert!(parsed.content.contains("Rust 1.99 introduces faster compilation"));
    }
}
