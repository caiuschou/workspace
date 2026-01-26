use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ResourceContentsBase;

/// Blob resource contents encoded as base64.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BlobResourceContents {
    #[serde(flatten)]
    pub base: ResourceContentsBase,
    pub blob: String,
}
