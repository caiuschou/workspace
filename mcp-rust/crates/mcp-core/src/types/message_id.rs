use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Unique identifier used for requests and responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum MessageId {
    String(String),
    Number(i64),
}

impl MessageId {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            MessageId::String(value) => Some(value),
            MessageId::Number(_) => None,
        }
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageId::String(value) => write!(f, "{value}"),
            MessageId::Number(value) => write!(f, "{value}"),
        }
    }
}

impl From<&str> for MessageId {
    fn from(value: &str) -> Self {
        MessageId::String(value.to_string())
    }
}

impl From<String> for MessageId {
    fn from(value: String) -> Self {
        MessageId::String(value)
    }
}

impl From<i64> for MessageId {
    fn from(value: i64) -> Self {
        MessageId::Number(value)
    }
}
