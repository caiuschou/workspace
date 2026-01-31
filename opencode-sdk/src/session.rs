//! Session API for OpenCode Server.
//!
//! Create sessions and send messages to AI assistants.

use crate::client::Client;
use crate::Error;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Parses message list from API response. Handles wrapped format {messages: [...]}.
fn parse_message_list(body: &str) -> Result<Vec<MessageListItem>, Error> {
    let wrapped: serde_json::Value = serde_json::from_str(body)?;
    let raw: Vec<serde_json::Value> = wrapped
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .or_else(|| wrapped.as_array().cloned())
        .unwrap_or_default();

    let mut items = Vec::new();
    for v in raw {
        if let Ok(item) = parse_message_item(&v) {
            items.push(item);
        }
    }
    Ok(items)
}

fn parse_message_item(v: &serde_json::Value) -> Result<MessageListItem, Error> {
    let (id, role) = if let Some(info) = v.get("info") {
        (
            info.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            info.get("role").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        )
    } else {
        (
            v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            v.get("role").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        )
    };

    let parts = v
        .get("parts")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| serde_json::from_value(p.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(MessageListItem {
        info: MessageInfo { id, role },
        parts,
    })
}

/// Session created by the server.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    /// Part ID (e.g. "prt_...").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Part type: "text", "reasoning", "image", "binary", "tool_call", "tool_result", "finish", etc.
    #[serde(rename = "type")]
    pub part_type: String,
    /// Text content for text parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Reasoning content (ReasoningContent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Image URL for image parts (ImageURLContent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Raw/binary content (BinaryContent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    /// Tool name for tool call/result parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool input for tool call parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
    /// Tool output for tool result parts.
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
    /// Creates a text part.
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

/// Request body for creating a session.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    /// Optional session title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Request body for sending a message.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    /// Message parts (required).
    pub parts: Vec<Part>,
}

/// Message in a session (minimal for polling).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    /// Message ID.
    pub id: String,
    /// Role: "user" or "assistant".
    pub role: String,
}

/// Response item from GET /session/{id}/message.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageListItem {
    /// Message metadata.
    pub info: MessageInfo,
    /// Message parts (text, tool calls, etc.).
    #[serde(default)]
    pub parts: Vec<Part>,
}

impl MessageListItem {
    /// Extracts all text from parts, concatenated.
    pub fn text_content(&self) -> String {
        self.parts
            .iter()
            .filter_map(|p| p.text.as_deref())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Max length of content preview in receive logs.
const PART_PREVIEW_LEN: usize = 120;

/// Logs a single part when received (type, id, and brief content summary).
/// Uses DEBUG level so that when the caller is polling (e.g. wait_for_assistant_response),
/// INFO only shows "received message list count=N" per poll; set RUST_LOG=debug to see each part.
fn log_part_received(index: usize, part: &Part) {
    let id = part.id.as_deref().unwrap_or("-");
    let ty = part.part_type.as_str();
    match ty {
        "text" => {
            let len = part.text.as_ref().map(|s| s.len()).unwrap_or(0);
            let preview = part
                .text
                .as_deref()
                .map(|s| {
                    if s.len() <= PART_PREVIEW_LEN {
                        s.to_string()
                    } else {
                        format!("{}...", s.chars().take(PART_PREVIEW_LEN).collect::<String>())
                    }
                })
                .unwrap_or_default();
            debug!(
                part_index = index,
                part_id = %id,
                part_type = %ty,
                len = len,
                preview = %preview,
                "received part: text"
            );
        }
        "reasoning" => {
            let len = part.reasoning.as_ref().map(|s| s.len()).unwrap_or(0);
            let preview = part
                .reasoning
                .as_deref()
                .map(|s| {
                    if s.len() <= PART_PREVIEW_LEN {
                        s.to_string()
                    } else {
                        format!("{}...", s.chars().take(PART_PREVIEW_LEN).collect::<String>())
                    }
                })
                .unwrap_or_default();
            debug!(
                part_index = index,
                part_id = %id,
                part_type = %ty,
                len = len,
                preview = %preview,
                "received part: reasoning"
            );
        }
        "image" | "image_url" => {
            debug!(part_index = index, part_id = %id, part_type = %ty, url = ?part.image_url, "received part: image");
        }
        "binary" => {
            debug!(part_index = index, part_id = %id, part_type = %ty, "received part: binary");
        }
        "tool_call" | "tool" => {
            debug!(
                part_index = index,
                part_id = %id,
                part_type = %ty,
                tool_name = ?part.tool_name,
                finished = ?part.finished,
                "received part: tool call"
            );
        }
        "tool_result" => {
            debug!(
                part_index = index,
                part_id = %id,
                part_type = %ty,
                tool_name = ?part.tool_name,
                tool_call_id = ?part.tool_call_id,
                is_error = ?part.is_error,
                "received part: tool result"
            );
        }
        "finish" => {
            debug!(part_index = index, part_id = %id, part_type = %ty, finish_reason = ?part.finish_reason, "received part: finish");
        }
        "step-start" => {
            debug!(part_index = index, part_id = %id, part_type = %ty, "received part: step-start");
        }
        "step-finish" => {
            debug!(part_index = index, part_id = %id, part_type = %ty, "received part: step-finish");
        }
        _ => {
            debug!(part_index = index, part_id = %id, part_type = %ty, "received part: unknown type");
        }
    }
}

impl Client {
    /// Creates a new session, optionally in the given project directory.
    ///
    /// # Arguments
    ///
    /// * `directory` - Project directory (absolute path). Omit to use server's cwd.
    pub async fn session_create(
        &self,
        directory: Option<&std::path::Path>,
        request: CreateSessionRequest,
    ) -> Result<Session, Error> {
        let url = format!("{}/session", self.base_url());
        let mut req = self
            .http()
            .post(&url)
            .json(&request);

        if let Some(dir) = directory {
            if let Some(s) = dir.to_str() {
                req = req.query(&[("directory", s)]);
            }
        }

        let response = req.send().await?;
        let session: Session = response.json().await?;
        Ok(session)
    }

    /// Sends a message to a session (async, returns immediately).
    ///
    /// Uses `POST /session/{id}/prompt_async` - the AI response is processed
    /// in the background. Use `session_send_message` for streaming response.
    pub async fn session_send_message_async(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        request: SendMessageRequest,
    ) -> Result<(), Error> {
        let url = format!("{}/session/{}/prompt_async", self.base_url(), session_id);
        let mut req = self
            .http()
            .post(&url)
            .json(&request);

        if let Some(dir) = directory {
            if let Some(s) = dir.to_str() {
                req = req.query(&[("directory", s)]);
            }
        }

        req.send().await?.error_for_status()?;
        Ok(())
    }

    /// Lists messages in a session.
    ///
    /// Tries both /message and /messages as different OpenCode versions may use either.
    /// When used for polling (e.g. `wait_for_assistant_response` in open.rs), this is called
    /// every 2 seconds until the assistant message has content; each call logs "received message list".
    pub async fn session_list_messages(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Vec<MessageListItem>, Error> {
        let path = format!("{}/session/{}/message", self.base_url(), session_id);
        let mut items = self.session_list_messages_at(&path, directory).await;

        if items.as_ref().map(|v| v.is_empty()).unwrap_or(true) {
            let path_plural = format!("{}/session/{}/messages", self.base_url(), session_id);
            if let Ok(plural_items) = self.session_list_messages_at(&path_plural, directory).await {
                if !plural_items.is_empty() {
                    items = Ok(plural_items);
                }
            }
        }
        if items.as_ref().map(|v| v.is_empty()).unwrap_or(true) && directory.is_some() {
            if let Ok(no_dir) = self.session_list_messages_at(&path, None).await {
                if !no_dir.is_empty() {
                    items = Ok(no_dir);
                }
            }
        }
        items
    }

    async fn session_list_messages_at(
        &self,
        url: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Vec<MessageListItem>, Error> {
        let mut req = self.http().get(url);

        if let Some(dir) = directory {
            if let Some(s) = dir.to_str() {
                req = req.query(&[("directory", s)]);
            }
        }

        let response = req.send().await?;
        let body = response.text().await?;

        let items: Vec<MessageListItem> = serde_json::from_str(&body).unwrap_or_else(|_| {
            debug!("fallback parse_message_list");
            parse_message_list(&body).unwrap_or_default()
        });
        // "received message list": we just fetched the session's message list from the server
        // (GET /session/{id}/message or /messages). This log can appear multiple times in sequence
        // when the caller is polling (e.g. wait_for_assistant_response in open.rs polls every 2s until
        // the assistant message has content); each poll is a separate list fetch, so the same list is
        // "received" repeatedly until the assistant reply is ready.
        info!(count = items.len(), "received message list");
        for (msg_index, item) in items.iter().enumerate() {
            debug!(
                msg_index,
                message_id = %item.info.id,
                role = %item.info.role,
                parts_count = item.parts.len(),
                "received message"
            );
            for (part_index, part) in item.parts.iter().enumerate() {
                log_part_received(part_index, part);
            }
        }
        Ok(items)
    }
}
