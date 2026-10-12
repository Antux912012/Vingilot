//! Vingilot AI integration, providers, and MCP tools.

pub mod dispatcher;
pub mod mcp;
pub mod provider;

pub use dispatcher::{BrowserToolDefinition, BrowserToolDispatcher, ToolRisk};
pub use mcp::{McpServer, McpTool};
pub use provider::{
    AiError, AiProvider, AiResult, ChatCompletionResponse, ChatMessage, OpenAiCompatibleProvider,
};
