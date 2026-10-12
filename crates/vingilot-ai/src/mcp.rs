use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use vingilot_engine::WebKitEngine;

use crate::dispatcher::BrowserToolDispatcher;

/// MCP tool schema representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Model Context Protocol (MCP) server exposing Vingilot browser tools over stdio/JSON-RPC.
pub struct McpServer {
    dispatcher: BrowserToolDispatcher,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            dispatcher: BrowserToolDispatcher::new(),
        }
    }

    /// Returns list of registered tools conforming to MCP specifications.
    pub fn list_tools(&self) -> Vec<McpTool> {
        self.dispatcher
            .tools()
            .iter()
            .map(|t| McpTool {
                name: t.name.clone(),
                description: format!("{} [Risk: {:?}]", t.description, t.risk),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": true
                }),
            })
            .collect()
    }

    /// Handles an incoming MCP `tools/call` JSON-RPC style request.
    pub fn handle_tool_call(
        &self,
        engine: &mut WebKitEngine,
        tool_name: &str,
        arguments: &Value,
        confirmed: bool,
    ) -> Value {
        match self
            .dispatcher
            .dispatch(engine, tool_name, arguments, confirmed)
        {
            Ok(output) => json!({
                "content": [{
                    "type": "text",
                    "text": output
                }],
                "isError": false
            }),
            Err(err) => json!({
                "content": [{
                    "type": "text",
                    "text": err
                }],
                "isError": true
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_listing_and_execution() {
        let server = McpServer::new();
        let tools = server.list_tools();
        assert!(tools.len() >= 12);
        assert!(tools.iter().any(|t| t.name == "get_current_url"));

        let mut engine = WebKitEngine::default();
        let resp = server.handle_tool_call(&mut engine, "get_current_url", &json!({}), false);
        assert_eq!(resp["isError"], false);
        assert_eq!(resp["content"][0]["text"], "about:blank");
    }
}
