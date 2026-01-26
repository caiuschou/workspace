use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Task creation parameters, used to ask that the server create a task to represent a request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct TaskCreationParams {
    /// Time in milliseconds to keep task results available after completion.
    /// If null, the task has unlimited lifetime until manually cleaned up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    /// Time in milliseconds to wait between task status requests.
    #[serde(rename = "pollInterval", skip_serializing_if = "Option::is_none")]
    pub poll_interval: Option<u64>,
}
