use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ContentBlock;

/// The result of a tool execution, provided by the user (server).
/// Represents the outcome of invoking a tool requested via ToolUseContent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ToolResultContent {
    #[serde(rename = "type")]
    pub kind: String,
    /// The unique identifier for the corresponding tool call.
    #[serde(rename = "toolUseId")]
    pub tool_use_id: String,
    /// Content blocks representing the tool result.
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    /// Optional structured content matching the tool's output schema.
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
    /// Whether the tool execution resulted in an error.
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

impl ToolResultContent {
    pub fn new(tool_use_id: impl Into<String>, content: Vec<ContentBlock>) -> Self {
        Self {
            kind: "tool_result".to_string(),
            tool_use_id: tool_use_id.into(),
            content,
            structured_content: None,
            is_error: None,
            meta: None,
        }
    }

    pub fn error(tool_use_id: impl Into<String>, content: Vec<ContentBlock>) -> Self {
        Self {
            kind: "tool_result".to_string(),
            tool_use_id: tool_use_id.into(),
            content,
            structured_content: None,
            is_error: Some(true),
            meta: None,
        }
    }
}
