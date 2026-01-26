use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{PaginatedResult, Resource};

/// Result for resources/list.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ListResourcesResult {
    #[serde(flatten)]
    pub pagination: PaginatedResult,
    pub resources: Vec<Resource>,
}
