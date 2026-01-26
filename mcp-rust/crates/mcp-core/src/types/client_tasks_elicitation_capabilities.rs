use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CapabilityFlag;

/// Task support for client elicitation requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ClientTasksElicitationCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create: Option<CapabilityFlag>,
}
