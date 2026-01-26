use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Task;

/// Result for tasks/cancel.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CancelTaskResult {
    #[serde(flatten)]
    pub task: Task,
}
