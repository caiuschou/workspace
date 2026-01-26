use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ClientCapabilities, Implementation, RequestParams};

/// Parameters for the initialize request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct InitializeRequestParams {
    #[serde(flatten)]
    pub base: RequestParams,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
}
