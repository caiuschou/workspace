use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Task;

/// Parameters for notifications/tasks/status.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TaskStatusNotificationParams {
    #[serde(flatten)]
    pub task: Task,
}
