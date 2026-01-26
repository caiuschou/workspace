use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ResourceContentsBase;

/// Text resource contents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TextResourceContents {
    #[serde(flatten)]
    pub base: ResourceContentsBase,
    pub text: String,
}
