use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CapabilityFlag, ClientTasksRequestCapabilities};

/// Task capabilities for clients, indicating which request types support task creation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ClientTasksCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<ClientTasksRequestCapabilities>,
}
