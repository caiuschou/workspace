use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CapabilityFlag, ServerTasksRequestCapabilities};

/// Task capabilities for servers, indicating which request types support task creation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ServerTasksCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<ServerTasksRequestCapabilities>,
}
