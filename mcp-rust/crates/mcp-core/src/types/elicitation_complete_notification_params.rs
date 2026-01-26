use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parameters for a `notifications/elicitation/complete` notification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElicitationCompleteNotificationParams {
    /// The ID of the elicitation that completed.
    #[serde(rename = "elicitationId")]
    pub elicitation_id: String,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

impl ElicitationCompleteNotificationParams {
    pub fn new(elicitation_id: impl Into<String>) -> Self {
        Self {
            elicitation_id: elicitation_id.into(),
            meta: None,
        }
    }
}
