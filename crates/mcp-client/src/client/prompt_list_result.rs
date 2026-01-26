use serde::{Deserialize, Serialize};

use crate::client::PromptDefinition;

/// Result payload for prompts/list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PromptListResult {
    pub prompts: Vec<PromptDefinition>,
}
