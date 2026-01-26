use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CapabilityFlag;

/// Task support for tool requests on servers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ServerTasksToolCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<CapabilityFlag>,
}
