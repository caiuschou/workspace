use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result payload for tasks/result.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TaskResult {
    pub task: Value,
}
