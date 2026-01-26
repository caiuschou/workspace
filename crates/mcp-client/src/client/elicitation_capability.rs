use serde::{Deserialize, Serialize};

use crate::client::{CapabilityFlag, ElicitationFormCapability};

/// Elicitation capability configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ElicitationCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<ElicitationFormCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<CapabilityFlag>,
}
