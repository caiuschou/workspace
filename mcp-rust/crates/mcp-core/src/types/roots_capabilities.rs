use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Capabilities for client roots support.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct RootsCapabilities {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}
