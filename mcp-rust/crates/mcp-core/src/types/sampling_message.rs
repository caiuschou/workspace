use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Role, SamplingMessageContent};

/// Describes a message issued to or received from an LLM API.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SamplingMessage {
    pub role: Role,
    /// Content can be a single block or an array of blocks.
    pub content: SamplingMessageContentOrArray,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

/// Content can be a single block or an array of blocks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum SamplingMessageContentOrArray {
    Single(SamplingMessageContent),
    Array(Vec<SamplingMessageContent>),
}

impl SamplingMessage {
    pub fn user(content: impl Into<SamplingMessageContent>) -> Self {
        Self {
            role: Role::User,
            content: SamplingMessageContentOrArray::Single(content.into()),
            meta: None,
        }
    }

    pub fn assistant(content: impl Into<SamplingMessageContent>) -> Self {
        Self {
            role: Role::Assistant,
            content: SamplingMessageContentOrArray::Single(content.into()),
            meta: None,
        }
    }

    pub fn user_multi(content: Vec<SamplingMessageContent>) -> Self {
        Self {
            role: Role::User,
            content: SamplingMessageContentOrArray::Array(content),
            meta: None,
        }
    }

    pub fn assistant_multi(content: Vec<SamplingMessageContent>) -> Self {
        Self {
            role: Role::Assistant,
            content: SamplingMessageContentOrArray::Array(content),
            meta: None,
        }
    }
}
