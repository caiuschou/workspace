use serde::{Deserialize, Serialize};

use crate::client::{
    CapabilityFlag, ClientTasksCapability, ElicitationCapability, RootsCapability,
};

/// Flags describing what the client can do.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<ElicitationCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<ClientTasksCapability>,
}

impl ClientCapabilities {
    pub fn merge(&self, other: &ClientCapabilities) -> ClientCapabilities {
        ClientCapabilities {
            roots: other.roots.clone().or_else(|| self.roots.clone()),
            sampling: other.sampling.clone().or_else(|| self.sampling.clone()),
            elicitation: other
                .elicitation
                .clone()
                .or_else(|| self.elicitation.clone()),
            tasks: other.tasks.clone().or_else(|| self.tasks.clone()),
        }
    }
}
