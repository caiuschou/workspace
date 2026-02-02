//! Session and message request/response types (Serve API 08-session).

use serde::{Deserialize, Serialize};

/// Session created by the server (from `POST /session`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Session ID (e.g. "ses_...").
    pub id: String,
    /// Optional title.
    #[serde(default)]
    pub title: Option<String>,
}

/// Part of a message. Aligns with server ContentPart: text, reasoning, image, binary, tool call, tool result, finish.
///
/// **Request:** Create text parts with [`Part::text`]. Use in [`SendMessageRequest::parts`].
///
/// **Response (agent reply):** Each part has a `part_type`; only some fields are set per type:
///
/// | `part_type`     | Meaning        | Relevant fields                          |
/// |-----------------|----------------|------------------------------------------|
/// | `"text"`        | Plain text     | `text`                                   |
/// | `"reasoning"`   | Reasoning step| `reasoning`                              |
/// | `"image"`      | Image          | `image_url`                              |
/// | `"tool"` / `"tool_call"` / `"tool_result"` | Tool usage | `tool_name`, `tool_input`, `tool_output` |
/// | `"finish"`     | Turn end       | `finish_reason`                          |
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    /// Part ID (e.g. "prt_...").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Part type: "text", "reasoning", "image", "binary", "tool", "tool_call", "tool_result", "finish", etc.
    #[serde(rename = "type")]
    pub part_type: String,
    /// Text content. Set for `part_type == "text"`. [`MessageListItem::text_content`] concatenates these.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Reasoning content (ReasoningContent). Set for `part_type == "reasoning"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Image URL for image parts (ImageURLContent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Raw/binary content (BinaryContent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    /// Tool name. Set for tool_call / tool_result parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool input (arguments). Set for tool_call parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
    /// Tool output (result). Set for tool_result parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<serde_json::Value>,
    /// Tool call ID (for ToolResult).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Whether the tool call is finished (for ToolCall).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished: Option<bool>,
    /// Metadata (e.g. for ToolResult).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Whether the tool result is an error (for ToolResult).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Finish reason: "end_turn", "max_tokens", "tool_use", "canceled", "error", "permission_denied" (for Finish).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

impl Part {
    /// Creates a text part for use in [`SendMessageRequest`].
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            id: None,
            part_type: "text".to_string(),
            text: Some(content.into()),
            reasoning: None,
            image_url: None,
            content: None,
            tool_name: None,
            tool_input: None,
            tool_output: None,
            tool_call_id: None,
            finished: None,
            metadata: None,
            is_error: None,
            finish_reason: None,
        }
    }
}

/// Request body for `POST /session` (create session).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    /// Optional parent session ID (pattern: ^ses.*).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Optional session title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional permission ruleset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<serde_json::Value>,
}

/// Request body for `POST /session/{id}/prompt_async` and `POST /session/{id}/message`.
///
/// Create with [`SendMessageRequest::from_parts`] using [`Part::text`] parts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    /// Message parts (required).
    pub parts: Vec<Part>,
    /// Optional message ID (pattern: ^msg.*).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Optional model override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<serde_json::Value>,
    /// Optional agent override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Optional no-reply flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_reply: Option<bool>,
    /// Optional system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Optional variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

impl SendMessageRequest {
    /// Creates a request with the given parts; optional fields are `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use opencode_sdk::{SendMessageRequest, Part};
    ///
    /// let req = SendMessageRequest::from_parts(vec![Part::text("Hello")]);
    /// ```
    pub fn from_parts(parts: Vec<Part>) -> Self {
        Self {
            parts,
            message_id: None,
            model: None,
            agent: None,
            no_reply: None,
            system: None,
            variant: None,
        }
    }
}

/// Message metadata in a session (id, role).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    /// Message ID.
    pub id: String,
    /// Role: "user" or "assistant".
    pub role: String,
}

/// Response item from `GET /session/{id}/message` or `/messages`.
///
/// Used as assistant reply content: when you send a message and wait for the agent,
/// the last assistant message is returned as [`OpenResult::assistant_reply`](crate::OpenResult::assistant_reply).
/// Each message has `info` (id, role) and `parts` (text, tool calls, tool results, etc.).
///
/// Use [`text_content`](Self::text_content) to get all text from parts concatenated.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageListItem {
    /// Message metadata (id, role).
    pub info: MessageInfo,
    /// Response parts: text segments and tool call/result segments in order.
    #[serde(default)]
    pub parts: Vec<Part>,
}

impl MessageListItem {
    /// Extracts all text from parts (only `part.text`), concatenated with newlines.
    /// Does not include `reasoning` or other non-text fields.
    pub fn text_content(&self) -> String {
        self.parts
            .iter()
            .filter_map(|p| p.text.as_deref())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Query params for listing sessions (`GET /session`).
#[derive(Debug, Clone, Default)]
pub struct SessionListParams {
    /// Only return root sessions (no parentID).
    pub roots: Option<bool>,
    /// Filter sessions updated on or after this timestamp (ms since epoch).
    pub start: Option<i64>,
    /// Filter by title (case-insensitive).
    pub search: Option<String>,
    /// Max number of sessions to return.
    pub limit: Option<u32>,
}
