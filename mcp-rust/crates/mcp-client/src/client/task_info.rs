use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Task information emitted by task-aware responses.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TaskInfo {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}
