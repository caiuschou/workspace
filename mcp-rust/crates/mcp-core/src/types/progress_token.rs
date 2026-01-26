use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A progress token used to correlate progress notifications with requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ProgressToken {
    String(String),
    Number(i64),
}

impl From<&str> for ProgressToken {
    fn from(value: &str) -> Self {
        ProgressToken::String(value.to_string())
    }
}

impl From<String> for ProgressToken {
    fn from(value: String) -> Self {
        ProgressToken::String(value)
    }
}

impl From<i64> for ProgressToken {
    fn from(value: i64) -> Self {
        ProgressToken::Number(value)
    }
}
