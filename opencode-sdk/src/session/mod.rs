//! Session API for OpenCode Server.
//!
//! Create sessions and send messages to AI assistants.

mod diff;
mod message;
mod types;

pub use diff::DiffItem;
pub use types::{
    CreateSessionRequest, MessageInfo, MessageListItem, Part, SendMessageRequest, Session,
    SessionListParams,
};

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use tracing::{debug, info};

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
    /// Lists sessions, optionally filtered by directory, roots, start, search, limit.
    ///
    /// `GET /session`
    pub async fn session_list(
        &self,
        directory: Option<&std::path::Path>,
        params: Option<SessionListParams>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/session", self.base_url());
        let mut req = self.http().get(&url).with_directory(directory);

        if let Some(p) = params {
            if let Some(v) = p.roots {
                req = req.query(&[("roots", v.to_string())]);
            }
            if let Some(v) = p.start {
                req = req.query(&[("start", v.to_string())]);
            }
            if let Some(ref v) = p.search {
                req = req.query(&[("search", v)]);
            }
            if let Some(v) = p.limit {
                req = req.query(&[("limit", v.to_string())]);
            }
        }

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<serde_json::Value> =
            serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }

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
        let req = self
            .http()
            .post(&url)
            .json(&request)
            .with_directory(directory);

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
        let req = self
            .http()
            .post(&url)
            .json(&request)
            .with_directory(directory);

        req.send().await?.error_for_status()?;
        Ok(())
    }

    /// Lists messages in a session.
    ///
    /// Tries multiple (path, directory) combinations: /message and /messages (different
    /// OpenCode versions), and without directory when directory is set. First non-empty
    /// result is returned.
    pub async fn session_list_messages(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Vec<MessageListItem>, Error> {
        let base = format!("{}/session/{}", self.base_url(), session_id);
        let mut tries: Vec<(&str, Option<&std::path::Path>)> = vec![
            ("/message", directory),
            ("/messages", directory),
        ];
        if directory.is_some() {
            tries.push(("/message", None));
        }
        let mut last = self.session_list_messages_at(&format!("{}{}", base, tries[0].0), tries[0].1).await;
        for (suffix, dir) in tries.iter().skip(1) {
            if last.as_ref().map(|v| !v.is_empty()).unwrap_or(false) {
                return last;
            }
            let path = format!("{}{}", base, suffix);
            if let Ok(items) = self.session_list_messages_at(&path, *dir).await {
                if !items.is_empty() {
                    return Ok(items);
                }
                last = Ok(items);
            }
        }
        last
    }

    /// Gets the file changes (diff) for a session, optionally for a specific message.
    ///
    /// `GET /session/{sessionID}/diff`. Returns an array of diff items.
    pub async fn session_diff(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        message_id: Option<&str>,
    ) -> Result<Vec<DiffItem>, Error> {
        let url = format!("{}/session/{}/diff", self.base_url(), session_id);
        let mut req = self.http().get(&url).with_directory(directory);

        if let Some(msg_id) = message_id {
            req = req.query(&[("messageID", msg_id)]);
        }

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<DiffItem> = serde_json::from_str(&body).unwrap_or_else(|_| {
            debug!("session_diff: response not JSON, wrapping as empty array");
            Vec::new()
        });
        Ok(value)
    }

    /// Gets status of all sessions (active, idle, completed).
    ///
    /// `GET /session/status`
    pub async fn session_status(
        &self,
        directory: Option<&std::path::Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/session/status", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Gets a session by ID.
    ///
    /// `GET /session/{sessionID}`
    pub async fn session_get(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Session, Error> {
        let url = format!("{}/session/{}", self.base_url(), session_id);
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Deletes a session and all associated data.
    ///
    /// `DELETE /session/{sessionID}`
    pub async fn session_delete(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<bool, Error> {
        let url = format!("{}/session/{}", self.base_url(), session_id);
        let req = self.http().delete(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Updates session properties (e.g. title).
    ///
    /// `PATCH /session/{sessionID}`
    pub async fn session_update(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        body: impl serde::Serialize,
    ) -> Result<Session, Error> {
        let url = format!("{}/session/{}", self.base_url(), session_id);
        let req = self.http().patch(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Gets child sessions forked from this session.
    ///
    /// `GET /session/{sessionID}/children`
    pub async fn session_children(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/session/{}/children", self.base_url(), session_id);
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }

    /// Gets the todo list for a session.
    ///
    /// `GET /session/{sessionID}/todo`
    pub async fn session_todo(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/session/{}/todo", self.base_url(), session_id);
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }

    /// Initializes a session (analyzes project, generates AGENTS.md).
    ///
    /// `POST /session/{sessionID}/init`
    pub async fn session_init(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        provider_id: &str,
        model_id: &str,
        message_id: &str,
    ) -> Result<bool, Error> {
        let url = format!("{}/session/{}/init", self.base_url(), session_id);
        let req = self
            .http()
            .post(&url)
            .json(&serde_json::json!({
                "providerID": provider_id,
                "modelID": model_id,
                "messageID": message_id
            }))
            .with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Forks a session at a message point.
    ///
    /// `POST /session/{sessionID}/fork`
    pub async fn session_fork(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        message_id: Option<&str>,
    ) -> Result<Session, Error> {
        let url = format!("{}/session/{}/fork", self.base_url(), session_id);
        let mut req = self.http().post(&url).with_directory(directory);
        if let Some(msg_id) = message_id {
            req = req.json(&serde_json::json!({ "messageID": msg_id }));
        } else {
            req = req.json(&serde_json::json!({}));
        }
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Aborts an active session.
    ///
    /// `POST /session/{sessionID}/abort`
    pub async fn session_abort(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<bool, Error> {
        let url = format!("{}/session/{}/abort", self.base_url(), session_id);
        let req = self.http().post(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Creates a shareable link for a session.
    ///
    /// `POST /session/{sessionID}/share`
    pub async fn session_share(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Session, Error> {
        let url = format!("{}/session/{}/share", self.base_url(), session_id);
        let req = self.http().post(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Removes shareable link for a session.
    ///
    /// `DELETE /session/{sessionID}/share`
    pub async fn session_unshare(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Session, Error> {
        let url = format!("{}/session/{}/share", self.base_url(), session_id);
        let req = self.http().delete(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Summarizes a session using AI.
    ///
    /// `POST /session/{sessionID}/summarize`
    pub async fn session_summarize(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        provider_id: &str,
        model_id: &str,
        auto: Option<bool>,
    ) -> Result<bool, Error> {
        let url = format!("{}/session/{}/summarize", self.base_url(), session_id);
        let mut body = serde_json::json!({ "providerID": provider_id, "modelID": model_id });
        if let Some(a) = auto {
            body["auto"] = serde_json::Value::Bool(a);
        }
        let req = self.http().post(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Sends a message and streams the AI response.
    ///
    /// `POST /session/{sessionID}/message`. Response is streamed; returns created message object.
    pub async fn session_send_message(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        request: SendMessageRequest,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/session/{}/message", self.base_url(), session_id);
        let req = self.http().post(&url).json(&request).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Gets a single message by ID.
    ///
    /// `GET /session/{sessionID}/message/{messageID}`
    pub async fn session_get_message(
        &self,
        session_id: &str,
        message_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!(
            "{}/session/{}/message/{}",
            self.base_url(),
            session_id,
            message_id
        );
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Deletes a part from a message.
    ///
    /// `DELETE /session/{sessionID}/message/{messageID}/part/{partID}`
    pub async fn session_delete_part(
        &self,
        session_id: &str,
        message_id: &str,
        part_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<bool, Error> {
        let url = format!(
            "{}/session/{}/message/{}/part/{}",
            self.base_url(),
            session_id,
            message_id,
            part_id
        );
        let req = self.http().delete(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Updates a part in a message.
    ///
    /// `PATCH /session/{sessionID}/message/{messageID}/part/{partID}`
    pub async fn session_update_part(
        &self,
        session_id: &str,
        message_id: &str,
        part_id: &str,
        directory: Option<&std::path::Path>,
        body: impl serde::Serialize,
    ) -> Result<Part, Error> {
        let url = format!(
            "{}/session/{}/message/{}/part/{}",
            self.base_url(),
            session_id,
            message_id,
            part_id
        );
        let req = self.http().patch(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Reverts a message in the session.
    ///
    /// `POST /session/{sessionID}/revert`
    pub async fn session_revert(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        message_id: &str,
        part_id: Option<&str>,
    ) -> Result<Session, Error> {
        let url = format!("{}/session/{}/revert", self.base_url(), session_id);
        let mut body = serde_json::json!({ "messageID": message_id });
        if let Some(pid) = part_id {
            body["partID"] = serde_json::Value::String(pid.to_string());
        }
        let req = self.http().post(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Restores all reverted messages.
    ///
    /// `POST /session/{sessionID}/unrevert`
    pub async fn session_unrevert(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Session, Error> {
        let url = format!("{}/session/{}/unrevert", self.base_url(), session_id);
        let req = self.http().post(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Responds to a permission request (approve/deny).
    ///
    /// `POST /session/{sessionID}/permissions/{permissionID}`
    pub async fn session_permission_respond(
        &self,
        session_id: &str,
        permission_id: &str,
        directory: Option<&std::path::Path>,
        response_value: &str,
    ) -> Result<bool, Error> {
        let url = format!(
            "{}/session/{}/permissions/{}",
            self.base_url(),
            session_id,
            permission_id
        );
        let req = self
            .http()
            .post(&url)
            .json(&serde_json::json!({ "response": response_value }))
            .with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Sends a command to a session for AI execution.
    ///
    /// `POST /session/{sessionID}/command`
    pub async fn session_command(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        body: impl serde::Serialize,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/session/{}/command", self.base_url(), session_id);
        let req = self.http().post(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Runs a shell command in the session context.
    ///
    /// `POST /session/{sessionID}/shell`
    pub async fn session_shell(
        &self,
        session_id: &str,
        directory: Option<&std::path::Path>,
        agent: &str,
        command: &str,
        model: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/session/{}/shell", self.base_url(), session_id);
        let mut body = serde_json::json!({ "agent": agent, "command": command });
        if let Some(m) = model {
            body["model"] = m;
        }
        let req = self.http().post(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    async fn session_list_messages_at(
        &self,
        url: &str,
        directory: Option<&std::path::Path>,
    ) -> Result<Vec<MessageListItem>, Error> {
        let req = self.http().get(url).with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;

        let items: Vec<MessageListItem> = serde_json::from_str(&body).unwrap_or_else(|_| {
            debug!("fallback parse_message_list");
            message::parse_message_list(&body).unwrap_or_default()
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
