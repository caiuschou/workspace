use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result payload for tools/call.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ToolCallResult {
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}
