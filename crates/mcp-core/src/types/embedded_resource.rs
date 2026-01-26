use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Annotations, ResourceContents};

/// Embedded resource content block.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EmbeddedResource {
    #[serde(rename = "type")]
    pub kind: String,
    pub resource: ResourceContents,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

impl EmbeddedResource {
    pub fn new(resource: ResourceContents) -> Self {
        Self {
            kind: "resource".to_string(),
            resource,
            annotations: None,
            meta: None,
        }
    }
}
