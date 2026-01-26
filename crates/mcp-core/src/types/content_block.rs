use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AudioContent, EmbeddedResource, ImageContent, ResourceLink, TextContent};

/// Content block used in prompts and tool results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum ContentBlock {
    Text(TextContent),
    Image(ImageContent),
    Audio(AudioContent),
    ResourceLink(ResourceLink),
    EmbeddedResource(EmbeddedResource),
}
