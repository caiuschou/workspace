use serde::{Deserialize, Serialize};

/// Marker type for capabilities represented as empty objects.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapabilityFlag {}
