use serde::{Deserialize, Serialize};

use crate::client::TaskInfo;

/// Result payload for tasks/list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TaskListResult {
    pub tasks: Vec<TaskInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
