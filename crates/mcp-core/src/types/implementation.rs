use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{BaseMetadata, Icons};

/// Name and version information for a client or server implementation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Implementation {
    #[serde(flatten)]
    pub base: BaseMetadata,
    #[serde(flatten)]
    pub icons: Icons,
    pub version: String,
    #[serde(rename = "websiteUrl", skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
