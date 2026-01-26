use serde::{Deserialize, Serialize};

use crate::client::TaskInfo;

/// Result payload for tasks/get.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TaskGetResult {
    pub task: TaskInfo,
}
