use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Role for prompt messages or annotations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}
