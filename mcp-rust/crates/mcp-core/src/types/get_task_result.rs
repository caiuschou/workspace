use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Task;

/// Result for tasks/get.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GetTaskResult {
    #[serde(flatten)]
    pub task: Task,
}
