use serde::{Deserialize, Serialize};

/// Execution metadata for a tool.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolExecution {
    #[serde(rename = "taskSupport", skip_serializing_if = "Option::is_none")]
    pub task_support: Option<String>,
}
