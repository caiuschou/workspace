use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ContentBlock, Role};

/// Message returned in prompt responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PromptMessage {
    pub role: Role,
    pub content: ContentBlock,
}
