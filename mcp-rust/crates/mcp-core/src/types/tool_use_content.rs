use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A tool call request from an assistant (LLM).
/// Represents the assistant's request to use a tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ToolUseContent {
    #[serde(rename = "type")]
    pub kind: String,
    /// The name of the tool to invoke.
    /// Must match a tool name from the request's tools array.
    pub name: String,
    /// Unique identifier for this tool call.
    /// Used to correlate with ToolResultContent in subsequent messages.
    pub id: String,
    /// Arguments to pass to the tool.
    /// Must conform to the tool's inputSchema.
    pub input: HashMap<String, Value>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

impl ToolUseContent {
    pub fn new(name: impl Into<String>, id: impl Into<String>, input: HashMap<String, Value>) -> Self {
        Self {
            kind: "tool_use".to_string(),
            name: name.into(),
            id: id.into(),
            input,
            meta: None,
        }
    }
}
