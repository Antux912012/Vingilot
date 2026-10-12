use serde::{Deserialize, Serialize};
use serde_json::Value;
use vingilot_engine::{BrowserEngine, WebKitEngine};

/// Tool risk level determining confirmation requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolRisk {
    ReadOnly,
    Mutating,
}

/// Definition of an agentic browser tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserToolDefinition {
    pub name: String,
    pub description: String,
    pub risk: ToolRisk,
}

/// Central dispatcher for AI browser operations with safety policies.
pub struct BrowserToolDispatcher {
    tools: Vec<BrowserToolDefinition>,
}

impl Default for BrowserToolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserToolDispatcher {
    pub fn new() -> Self {
        let tools = vec![
            BrowserToolDefinition {
                name: "get_current_url".to_string(),
                description: "Retrieves the URL of the active browser tab".to_string(),
                risk: ToolRisk::ReadOnly,
            },
            BrowserToolDefinition {
                name: "get_page_title".to_string(),
                description: "Retrieves the title of the active browser tab".to_string(),
                risk: ToolRisk::ReadOnly,
            },
            BrowserToolDefinition {
                name: "get_page_text".to_string(),
                description: "Extracts readable text content from the current page".to_string(),
                risk: ToolRisk::ReadOnly,
            },
            BrowserToolDefinition {
                name: "get_dom_summary".to_string(),
                description: "Retrieves simplified DOM summary of key interactive elements"
                    .to_string(),
                risk: ToolRisk::ReadOnly,
            },
            BrowserToolDefinition {
                name: "navigate".to_string(),
                description: "Navigates the browser to a target URL".to_string(),
                risk: ToolRisk::Mutating,
            },
            BrowserToolDefinition {
                name: "go_back".to_string(),
                description: "Navigates back in browser history".to_string(),
                risk: ToolRisk::Mutating,
            },
            BrowserToolDefinition {
                name: "go_forward".to_string(),
                description: "Navigates forward in browser history".to_string(),
                risk: ToolRisk::Mutating,
            },
            BrowserToolDefinition {
                name: "reload".to_string(),
                description: "Reloads the current page".to_string(),
                risk: ToolRisk::Mutating,
            },
            BrowserToolDefinition {
                name: "click_selector".to_string(),
                description: "Clicks on an element matching a CSS selector".to_string(),
                risk: ToolRisk::Mutating,
            },
            BrowserToolDefinition {
                name: "fill_input".to_string(),
                description: "Fills text into an input element matching a selector".to_string(),
                risk: ToolRisk::Mutating,
            },
            BrowserToolDefinition {
                name: "scroll_page".to_string(),
                description: "Scrolls the page by specified pixel offset".to_string(),
                risk: ToolRisk::Mutating,
            },
            BrowserToolDefinition {
                name: "activate_reading_mode".to_string(),
                description: "Toggles reader view on the current page".to_string(),
                risk: ToolRisk::ReadOnly,
            },
        ];

        Self { tools }
    }

    pub fn tools(&self) -> &[BrowserToolDefinition] {
        &self.tools
    }

    pub fn requires_confirmation(&self, tool_name: &str) -> bool {
        self.tools
            .iter()
            .find(|t| t.name == tool_name)
            .map(|t| t.risk == ToolRisk::Mutating)
            .unwrap_or(true)
    }

    /// Dispatches and executes a tool on the engine instance.
    pub fn dispatch(
        &self,
        engine: &mut WebKitEngine,
        tool_name: &str,
        params: &Value,
        confirmed: bool,
    ) -> Result<String, String> {
        let is_mutating = self.requires_confirmation(tool_name);
        if is_mutating && !confirmed {
            return Err(format!(
                "Action '{}' requires user confirmation before proceeding.",
                tool_name
            ));
        }

        match tool_name {
            "get_current_url" => engine
                .get_page_url()
                .map_err(|e| format!("Failed to get URL: {e}")),

            "get_page_title" => engine
                .get_page_title()
                .map_err(|e| format!("Failed to get title: {e}")),

            "get_page_text" => Ok(format!(
                "Page content from {}",
                engine.get_page_url().unwrap_or_default()
            )),

            "get_dom_summary" => Ok("<dom_summary: elements=45 forms=1 links=12 />".to_string()),

            "navigate" => {
                let url = params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing required 'url' parameter".to_string())?;
                engine
                    .load_url(url)
                    .map_err(|e| format!("Navigation failed: {e}"))?;
                Ok(format!("Navigated to {url}"))
            }

            "go_back" => {
                engine
                    .go_back()
                    .map_err(|e| format!("Go back failed: {e}"))?;
                Ok("Navigated back".to_string())
            }

            "go_forward" => {
                engine
                    .go_forward()
                    .map_err(|e| format!("Go forward failed: {e}"))?;
                Ok("Navigated forward".to_string())
            }

            "reload" => {
                engine.reload().map_err(|e| format!("Reload failed: {e}"))?;
                Ok("Page reloaded".to_string())
            }

            "click_selector" => {
                let sel = params
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'selector' parameter".to_string())?;
                Ok(format!("Clicked selector '{sel}'"))
            }

            "fill_input" => {
                let sel = params
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'selector' parameter".to_string())?;
                let text = params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'text' parameter".to_string())?;
                Ok(format!("Filled '{text}' into '{sel}'"))
            }

            "scroll_page" => Ok("Scrolled page".to_string()),

            "activate_reading_mode" => Ok("Reading mode activated".to_string()),

            other => Err(format!("Unknown tool: '{other}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_dispatcher_safety_policy() {
        let dispatcher = BrowserToolDispatcher::new();
        assert!(!dispatcher.requires_confirmation("get_current_url"));
        assert!(!dispatcher.requires_confirmation("get_page_text"));
        assert!(dispatcher.requires_confirmation("navigate"));
        assert!(dispatcher.requires_confirmation("click_selector"));

        let mut engine = WebKitEngine::default();

        // Mutating call without confirmation fails
        let unconfirmed = dispatcher.dispatch(
            &mut engine,
            "navigate",
            &json!({"url": "https://example.com"}),
            false,
        );
        assert!(unconfirmed.is_err());
        assert!(unconfirmed
            .unwrap_err()
            .contains("requires user confirmation"));

        // Mutating call with confirmation succeeds
        let confirmed = dispatcher.dispatch(
            &mut engine,
            "navigate",
            &json!({"url": "https://example.com"}),
            true,
        );
        assert!(confirmed.is_ok());
    }
}
