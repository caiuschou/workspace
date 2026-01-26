use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{PaginatedResult, Task};

/// Result for tasks/list.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ListTasksResult {
    #[serde(flatten)]
    pub pagination: PaginatedResult,
    pub tasks: Vec<Task>,
}
