//! Session API for OpenCode Server.
//!
//! Create sessions and send messages to AI assistants.

use crate::client::Client;
use crate::Error;
use serde::{Deserialize, Serialize};
use tracing::debug;

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

/// Part of a message (text, tool call, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    /// Part type (e.g. "text", "tool").
    #[serde(rename = "type")]
    pub part_type: String,
    /// Text content for text parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Tool name for tool parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool input for tool parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
    /// Tool output for tool parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<serde_json::Value>,
}

impl Part {
    /// Creates a text part.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            part_type: "text".to_string(),
            text: Some(content.into()),
            tool_name: None,
            tool_input: None,
            tool_output: None,
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
        debug!(count = items.len(), "session_list_messages");
        Ok(items)
    }
}
