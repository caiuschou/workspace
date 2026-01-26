//! Tool implementations

use mcp_core::types::{CallToolResult, ContentBlock, TextContent};

pub mod project;

/// Convert a result to MCP tool result
pub fn to_tool_result(content: String) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::Text(TextContent::new(content))],
        ..Default::default()
    }
}

/// Convert an error to MCP tool error result
pub fn to_tool_error(error: impl std::fmt::Display) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::Text(TextContent::new(format!("Error: {}", error)))],
        is_error: Some(true),
        ..Default::default()
    }
}
