use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Marker type for capabilities represented as empty objects.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct CapabilityFlag {}
