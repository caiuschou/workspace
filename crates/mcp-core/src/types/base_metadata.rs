use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Shared metadata fields for prompts, tools, resources, and implementations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BaseMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
