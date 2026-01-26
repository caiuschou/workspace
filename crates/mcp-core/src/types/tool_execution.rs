use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::TaskSupport;

/// Execution metadata for a tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ToolExecution {
    #[serde(rename = "taskSupport", skip_serializing_if = "Option::is_none")]
    pub task_support: Option<TaskSupport>,
}
