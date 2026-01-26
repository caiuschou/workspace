use serde::{Deserialize, Serialize};

/// Resource metadata returned by resources/list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResourceDefinition {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
