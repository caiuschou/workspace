use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Role, SamplingContent, SamplingMessageContent, StopReason};

/// The client's response to a sampling/createMessage request.
/// This is the backwards-compatible version that returns single content (no arrays).
/// Used when the request does not include tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CreateMessageResult {
    /// The name of the model that generated the message.
    pub model: String,
    /// The reason why sampling stopped, if known.
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    /// The role of the generated message.
    pub role: Role,
    /// Response content. Single content block (text, image, or audio).
    pub content: SamplingContent,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

/// The client's response to a sampling/createMessage request when tools were provided.
/// This version supports array content for tool use flows.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CreateMessageResultWithTools {
    /// The name of the model that generated the message.
    pub model: String,
    /// The reason why sampling stopped, if known.
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    /// The role of the generated message.
    pub role: Role,
    /// Response content. May be a single block or array. May include ToolUseContent if stopReason is "toolUse".
    pub content: CreateMessageContentOrArray,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

/// Content can be a single block or an array of blocks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum CreateMessageContentOrArray {
    Single(SamplingMessageContent),
    Array(Vec<SamplingMessageContent>),
}

impl CreateMessageResult {
    pub fn new(model: impl Into<String>, role: Role, content: SamplingContent) -> Self {
        Self {
            model: model.into(),
            stop_reason: None,
            role,
            content,
            meta: None,
        }
    }

    pub fn with_stop_reason(mut self, reason: StopReason) -> Self {
        self.stop_reason = Some(reason);
        self
    }
}

impl CreateMessageResultWithTools {
    pub fn new(model: impl Into<String>, role: Role, content: impl Into<CreateMessageContentOrArray>) -> Self {
        Self {
            model: model.into(),
            stop_reason: None,
            role,
            content: content.into(),
            meta: None,
        }
    }

    pub fn with_stop_reason(mut self, reason: StopReason) -> Self {
        self.stop_reason = Some(reason);
        self
    }
}

impl From<SamplingMessageContent> for CreateMessageContentOrArray {
    fn from(content: SamplingMessageContent) -> Self {
        CreateMessageContentOrArray::Single(content)
    }
}

impl From<Vec<SamplingMessageContent>> for CreateMessageContentOrArray {
    fn from(content: Vec<SamplingMessageContent>) -> Self {
        CreateMessageContentOrArray::Array(content)
    }
}
