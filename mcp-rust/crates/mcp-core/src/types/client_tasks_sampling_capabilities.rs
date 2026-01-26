use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CapabilityFlag;

/// Task support for client sampling requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ClientTasksSamplingCapabilities {
    #[serde(rename = "createMessage", skip_serializing_if = "Option::is_none")]
    pub create_message: Option<CapabilityFlag>,
}
