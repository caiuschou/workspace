use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Notification follows the same shape as a request but omits `id`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct NotificationMessage {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl NotificationMessage {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_owned(),
            method: method.into(),
            params,
        }
    }
}
