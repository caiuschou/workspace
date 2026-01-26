use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Metadata key used to associate requests/notifications with a related task.
pub const RELATED_TASK_META_KEY: &str = "io.modelcontextprotocol/related-task";

/// Metadata for associating messages with a task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RelatedTaskMetadata {
    #[serde(rename = "taskId")]
    pub task_id: String,
}
