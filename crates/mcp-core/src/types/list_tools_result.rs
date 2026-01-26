use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{PaginatedResult, Tool};

/// Result for tools/list.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ListToolsResult {
    #[serde(flatten)]
    pub pagination: PaginatedResult,
    pub tools: Vec<Tool>,
}
