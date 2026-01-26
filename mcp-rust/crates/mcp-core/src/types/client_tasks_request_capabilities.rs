use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ClientTasksElicitationCapabilities, ClientTasksSamplingCapabilities};

/// Task support for specific client request types.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ClientTasksRequestCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<ClientTasksSamplingCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<ClientTasksElicitationCapabilities>,
}
