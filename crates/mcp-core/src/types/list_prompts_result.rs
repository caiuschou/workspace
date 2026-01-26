use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{PaginatedResult, Prompt};

/// Result for prompts/list.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ListPromptsResult {
    #[serde(flatten)]
    pub pagination: PaginatedResult,
    pub prompts: Vec<Prompt>,
}
