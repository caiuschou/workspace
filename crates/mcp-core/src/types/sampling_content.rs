use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AudioContent, ImageContent, TextContent};

/// Basic content types for sampling responses (without tool use).
/// Used for backwards-compatible CreateMessageResult when tools are not used.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SamplingContent {
    Text(TextContent),
    Image(ImageContent),
    Audio(AudioContent),
}

impl From<TextContent> for SamplingContent {
    fn from(content: TextContent) -> Self {
        SamplingContent::Text(content)
    }
}

impl From<ImageContent> for SamplingContent {
    fn from(content: ImageContent) -> Self {
        SamplingContent::Image(content)
    }
}

impl From<AudioContent> for SamplingContent {
    fn from(content: AudioContent) -> Self {
        SamplingContent::Audio(content)
    }
}
