use serde::{Deserialize, Serialize};

use crate::client::{Implementation, ServerCapabilities};

/// Result returned by the initialize handshake.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}
