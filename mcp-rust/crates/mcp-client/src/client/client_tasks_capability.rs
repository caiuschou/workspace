use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Task capability configuration for clients.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct ClientTasksCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<Value>,
}
