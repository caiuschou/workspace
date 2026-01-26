use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{RequestMeta, Task};

/// Result returned when creating a task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CreateTaskResult {
    pub task: Task,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}
