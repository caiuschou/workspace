use serde::{Deserialize, Serialize};

/// Prompt metadata returned by prompts/list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PromptDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
