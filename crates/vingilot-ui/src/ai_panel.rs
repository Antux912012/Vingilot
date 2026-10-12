use vingilot_ai::{AiProvider, ChatMessage, OpenAiCompatibleProvider};

/// Interactive AI chat message displayed in the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChatEntry {
    pub is_user: bool,
    pub text: String,
}

/// AI Assistant sidebar panel providing chat, page summarization, and Q&A.
#[derive(Debug, Clone)]
pub struct AiAssistantPanel {
    is_open: bool,
    provider: OpenAiCompatibleProvider,
    history: Vec<AiChatEntry>,
}

impl Default for AiAssistantPanel {
    fn default() -> Self {
        Self::new("http://localhost:11434/v1", "llama3.2", None)
    }
}

impl AiAssistantPanel {
    pub fn new(base_url: &str, model: &str, api_key: Option<String>) -> Self {
        Self {
            is_open: false,
            provider: OpenAiCompatibleProvider::new(base_url, model, api_key),
            history: Vec::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn history(&self) -> &[AiChatEntry] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// User submits a query or instruction to the assistant.
    pub fn ask(
        &mut self,
        question: &str,
        current_page_context: Option<&str>,
    ) -> Result<String, String> {
        let mut prompt = question.to_string();
        if let Some(ctx) = current_page_context {
            prompt = format!("Context from active web page:\n{ctx}\n\nUser Question:\n{question}");
        }

        self.history.push(AiChatEntry {
            is_user: true,
            text: question.to_string(),
        });

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are the AI assistant inside the Vingilot web browser.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let response = self
            .provider
            .complete(&messages)
            .map_err(|e| format!("AI query failed: {e}"))?;

        self.history.push(AiChatEntry {
            is_user: false,
            text: response.content.clone(),
        });

        Ok(response.content)
    }

    /// Quick action to summarize current web page.
    pub fn summarize_page(
        &mut self,
        page_title: &str,
        page_content: &str,
    ) -> Result<String, String> {
        let question = format!("Summarize this page ('{page_title}') concisely.");
        self.ask(&question, Some(page_content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_assistant_panel() {
        let mut panel = AiAssistantPanel::default();
        assert!(!panel.is_open());
        panel.open();
        assert!(panel.is_open());

        let summary = panel
            .summarize_page(
                "Rust 1.99 Release",
                "Rust 1.99 features significant compilation improvements.",
            )
            .unwrap();

        assert!(summary.contains("Summary/Answer"));
        assert_eq!(panel.history().len(), 2);
        assert!(panel.history()[0].is_user);
        assert!(!panel.history()[1].is_user);
    }
}
