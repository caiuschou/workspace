use serde::{Deserialize, Serialize};

use crate::client::ToolDefinition;

/// Result payload for tools/list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ToolListResult {
    pub tools: Vec<ToolDefinition>,
}
