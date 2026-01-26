use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Capabilities describing form-based elicitation support.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct FormElicitationCapability {
    #[serde(rename = "applyDefaults", skip_serializing_if = "Option::is_none")]
    pub apply_defaults: Option<bool>,
    #[serde(flatten, default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, Value>,
}
