use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ServerTasksToolCapabilities;

/// Task support for specific server request types.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ServerTasksRequestCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ServerTasksToolCapabilities>,
}
