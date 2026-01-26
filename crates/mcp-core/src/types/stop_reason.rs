use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The reason why sampling stopped.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum StopReason {
    /// Natural end of the assistant's turn.
    #[serde(rename = "endTurn")]
    EndTurn,
    /// A stop sequence was encountered.
    #[serde(rename = "stopSequence")]
    StopSequence,
    /// Maximum token limit was reached.
    #[serde(rename = "maxTokens")]
    MaxTokens,
    /// The model wants to use one or more tools.
    #[serde(rename = "toolUse")]
    ToolUse,
    /// Provider-specific stop reason.
    #[serde(untagged)]
    Other(String),
}
