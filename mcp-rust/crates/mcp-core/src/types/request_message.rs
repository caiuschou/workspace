use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{JSONRPC_VERSION, MessageId};

/// JSON-RPC 2.0 style request payload.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RequestMessage {
    pub jsonrpc: String,
    pub id: MessageId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub params: Value,
}

impl RequestMessage {
    /// Creates a new request with the provided `id`.
    pub fn new(id: impl Into<MessageId>, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}
