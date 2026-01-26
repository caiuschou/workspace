use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AudioContent, ImageContent, TextContent, ToolResultContent, ToolUseContent};

/// Content block types allowed in sampling messages.
/// This includes text, image, audio, tool use requests, and tool results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type")]
pub enum SamplingMessageContent {
    #[serde(rename = "text")]
    Text(TextContent),
    #[serde(rename = "image")]
    Image(ImageContent),
    #[serde(rename = "audio")]
    Audio(AudioContent),
    #[serde(rename = "tool_use")]
    ToolUse(ToolUseContent),
    #[serde(rename = "tool_result")]
    ToolResult(ToolResultContent),
}

impl From<TextContent> for SamplingMessageContent {
    fn from(content: TextContent) -> Self {
        SamplingMessageContent::Text(content)
    }
}

impl From<ImageContent> for SamplingMessageContent {
    fn from(content: ImageContent) -> Self {
        SamplingMessageContent::Image(content)
    }
}

impl From<AudioContent> for SamplingMessageContent {
    fn from(content: AudioContent) -> Self {
        SamplingMessageContent::Audio(content)
    }
}

impl From<ToolUseContent> for SamplingMessageContent {
    fn from(content: ToolUseContent) -> Self {
        SamplingMessageContent::ToolUse(content)
    }
}

impl From<ToolResultContent> for SamplingMessageContent {
    fn from(content: ToolResultContent) -> Self {
        SamplingMessageContent::ToolResult(content)
    }
}
