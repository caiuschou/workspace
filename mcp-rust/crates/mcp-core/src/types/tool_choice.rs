use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Controls when tools are used in sampling requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    /// Model decides whether to use tools (default).
    Auto,
    /// Model MUST use at least one tool before completing.
    Required,
    /// Model MUST NOT use any tools.
    None,
}

impl Default for ToolChoiceMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// Controls tool usage behavior in sampling requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ToolChoice {
    /// Controls when tools are used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ToolChoiceMode>,
}

impl ToolChoice {
    pub fn auto() -> Self {
        Self {
            mode: Some(ToolChoiceMode::Auto),
        }
    }

    pub fn required() -> Self {
        Self {
            mode: Some(ToolChoiceMode::Required),
        }
    }

    pub fn none() -> Self {
        Self {
            mode: Some(ToolChoiceMode::None),
        }
    }
}
