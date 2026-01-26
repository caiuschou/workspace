use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{RequestMeta, TaskStatus};

/// Task state returned in task-related responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Task {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub status: TaskStatus,
    /// Time in milliseconds to keep task results available after completion.
    /// If null, the task has unlimited lifetime until manually cleaned up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    /// ISO 8601 timestamp when the task was created.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// ISO 8601 timestamp when the task was last updated.
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: String,
    /// Time in milliseconds to wait between task status requests.
    #[serde(rename = "pollInterval", skip_serializing_if = "Option::is_none")]
    pub poll_interval: Option<u64>,
    /// Optional diagnostic message for failed tasks or other status information.
    #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}
