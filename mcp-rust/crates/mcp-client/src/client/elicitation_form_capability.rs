use serde::{Deserialize, Serialize};

/// Form-mode elicitation options.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ElicitationFormCapability {
    #[serde(rename = "applyDefaults", skip_serializing_if = "Option::is_none")]
    pub apply_defaults: Option<bool>,
}
