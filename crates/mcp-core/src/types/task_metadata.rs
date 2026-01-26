use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Metadata for task creation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct TaskMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}
