use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ErrorObject, MessageId};

/// Response payload emitted by `Protocol`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ResultMessage {
    pub jsonrpc: String,
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObject>,
}

impl ResultMessage {
    pub fn success(id: impl Into<MessageId>, result: Value) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_owned(),
            id: id.into(),
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(id: impl Into<MessageId>, error: ErrorObject) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_owned(),
            id: id.into(),
            result: None,
            error: Some(error),
        }
    }
}
