use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CapabilityFlag;

/// Capabilities for client sampling support.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct SamplingCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<CapabilityFlag>,
}
