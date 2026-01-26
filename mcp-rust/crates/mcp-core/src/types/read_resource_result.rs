use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{RequestMeta, ResourceContents};

/// Result for resources/read.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContents>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}
