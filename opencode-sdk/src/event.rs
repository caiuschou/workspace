//! Event stream for real-time updates.
//!
//! Subscribes to GET /event (SSE) to receive streaming updates including
//! message.part.updated with text deltas.

use crate::client::Client;
use crate::Error;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::path::Path;
use tracing::{debug, info};

/// Streams events and invokes `on_text` for each text delta belonging to the session.
/// Returns when the stream ends or errors.
pub async fn subscribe_and_stream<F>(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    mut on_text: F,
) -> Result<(), Error>
where
    F: FnMut(&str) + Send,
{
    let url = format!("{}/event", client.base_url());
    let mut req = client.http().get(&url);
    if let Some(dir) = directory {
        if let Some(s) = dir.to_str() {
            req = req.query(&[("directory", s)]);
        }
    }
    let response = req
        .header("Accept", "text/event-stream")
        .timeout(std::time::Duration::from_secs(3600))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(Error::Http(response.error_for_status().unwrap_err()));
    }
    let mut stream = response.bytes_stream().eventsource();

    while let Some(result) = stream.next().await {
        match result {
            Ok(ev) => {
                if !ev.data.is_empty() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&ev.data) {
                        if let Ok(payload) = serde_json::to_string_pretty(&v) {
                            info!("event full payload:\n{}", payload);
                        } else {
                            info!("event full payload (raw): {}", ev.data);
                        }
                        if let Some(text) = extract_text_delta(&v, session_id) {
                            info!("stream chunk: {}", text);
                            on_text(&text);
                        } else {
                            debug!(event_type = ?v.get("type"), "event (no text delta for session)");
                        }
                    } else {
                        info!("event full payload (raw, not JSON): {}", ev.data);
                    }
                }
            }
            Err(e) => {
                debug!(error = %e, "event stream error, stopping");
                break;
            }
        }
    }
    Ok(())
}

/// Returns true if this event indicates the assistant turn is complete for the session.
///
/// Matches OpenCode server events per docs/opencode-serve-api/17-event-format.md:
/// - `session.idle`: session entered idle (recommended completion signal).
/// - `session.status` with `status.type === "idle"`.
/// - `message.part.updated` with `part.type === "step-finish"` or `"finish"`.
/// - `message.updated` with `info.finish` present.
/// - Legacy: event type contains "finish"/"complete", or properties.partType "finish".
pub(crate) fn extract_completion(v: &serde_json::Value, session_id: &str) -> bool {
    let ev_session = v
        .get("sessionID")
        .or(v.get("sessionId"))
        .or_else(|| v.get("properties").and_then(|p| p.get("sessionID").or(p.get("sessionId"))))
        .and_then(|s| s.as_str());
    if let Some(s) = ev_session {
        if s != session_id {
            return false;
        }
    }

    let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let props = v.get("properties").and_then(|p| p.as_object());

    // session.idle: recommended completion signal (session entered idle).
    if ty == "session.idle" {
        return true;
    }
    // session.status with status.type === "idle".
    if ty == "session.status" {
        if let Some(p) = props {
            let status_type = p
                .get("status")
                .and_then(|s| s.get("type"))
                .and_then(|t| t.as_str())
                .unwrap_or("");
            if status_type == "idle" {
                return true;
            }
        }
    }
    // message.part.updated with part.type === "step-finish" or "finish".
    if ty == "message.part.updated" {
        if let Some(p) = props {
            let part_type = p
                .get("part")
                .and_then(|part| part.get("type").and_then(|t| t.as_str()))
                .unwrap_or("");
            if part_type == "step-finish" || part_type == "finish" {
                return true;
            }
        }
    }
    // message.updated with info.finish (assistant message completed).
    if ty == "message.updated" {
        if let Some(p) = props {
            let info = match p.get("info") {
                Some(i) => i,
                None => return false,
            };
            if info.get("finish").is_some() {
                let info_session = info.get("sessionID").or(info.get("sessionId")).and_then(|s| s.as_str());
                if let Some(s) = info_session {
                    if s != session_id {
                        return false;
                    }
                }
                return true;
            }
        }
    }
    // Legacy / generic: event type or top-level properties.
    if ty.contains("finish") || ty.contains("complete") || ty == "done" {
        return true;
    }
    if let Some(p) = props {
        let part_type = p.get("partType").or(p.get("type")).and_then(|t| t.as_str()).unwrap_or("");
        if part_type == "finish" || part_type == "step-finish" {
            return true;
        }
        if p.get("finishReason").or(p.get("finish_reason")).is_some() {
            return true;
        }
    }
    false
}

/// Streams events, invokes `on_text` for each text delta, and returns when a completion
/// event is seen or the stream ends. Use this instead of polling to know when the
/// assistant reply is done.
pub async fn subscribe_and_stream_until_done<F>(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    mut on_text: F,
) -> Result<(), Error>
where
    F: FnMut(&str) + Send,
{
    let url = format!("{}/event", client.base_url());
    let mut req = client.http().get(&url);
    if let Some(dir) = directory {
        if let Some(s) = dir.to_str() {
            req = req.query(&[("directory", s)]);
        }
    }
    let response = req
        .header("Accept", "text/event-stream")
        .timeout(std::time::Duration::from_secs(3600))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(Error::Http(response.error_for_status().unwrap_err()));
    }
    let mut stream = response.bytes_stream().eventsource();

    while let Some(result) = stream.next().await {
        match result {
            Ok(ev) => {
                if !ev.data.is_empty() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&ev.data) {
                        // Log full event payload for debugging / inspecting OpenCode server format.
                        if let Ok(payload) = serde_json::to_string_pretty(&v) {
                            info!("event full payload:\n{}", payload);
                        } else {
                            info!("event full payload (raw): {}", ev.data);
                        }
                        if extract_completion(&v, session_id) {
                            info!("event stream: completion event received");
                            return Ok(());
                        }
                        if let Some(text) = extract_text_delta(&v, session_id) {
                            info!("stream chunk: {}", text);
                            on_text(&text);
                        } else {
                            debug!(event_type = ?v.get("type"), "event (no text delta for session)");
                        }
                    } else {
                        info!("event full payload (raw, not JSON): {}", ev.data);
                    }
                }
            }
            Err(e) => {
                debug!(error = %e, "event stream error, stopping");
                break;
            }
        }
    }
    info!("event stream ended");
    Ok(())
}

/// Extracts text delta from event JSON if it matches our session.
///
/// For OpenCode message.part.updated (see 17-event-format.md): prefers
/// `properties.delta` (streaming increment), then `properties.part.text`,
/// then `properties.text` / `properties.content`.
pub(crate) fn extract_text_delta(v: &serde_json::Value, session_id: &str) -> Option<String> {
    let props = v.get("properties").and_then(|p| p.as_object());
    let ev_session = v
        .get("sessionID")
        .or(v.get("sessionId"))
        .or_else(|| v.get("properties").and_then(|p| p.get("sessionID").or(p.get("sessionId"))))
        .and_then(|s| s.as_str());
    if let Some(s) = ev_session {
        if s != session_id {
            return None;
        }
    }
    // Session can also be inside properties.part.sessionID for message.part.updated.
    if let Some(p) = props {
        let part_session = p
            .get("part")
            .and_then(|part| part.get("sessionID").or(part.get("sessionId")).and_then(|s| s.as_str()));
        if let Some(s) = part_session {
            if s != session_id {
                return None;
            }
        }
        // Prefer delta (streaming increment) so we output only the new chunk.
        if let Some(t) = p.get("delta").and_then(|t| t.as_str()).filter(|s| !s.is_empty()) {
            return Some(t.to_string());
        }
        if let Some(t) = p.get("part").and_then(|part| part.get("text").and_then(|t| t.as_str())).filter(|s| !s.is_empty()) {
            return Some(t.to_string());
        }
        if let Some(t) = p.get("text").and_then(|t| t.as_str()).filter(|s| !s.is_empty()) {
            return Some(t.to_string());
        }
        if let Some(c) = p.get("content").and_then(|c| c.as_str()).filter(|s| !s.is_empty()) {
            return Some(c.to_string());
        }
    }
    let text = v
        .get("properties")
        .and_then(|p| p.get("text").and_then(|t| t.as_str()))
        .or_else(|| v.get("text").and_then(|t| t.as_str()))
        .or_else(|| v.get("properties").and_then(|p| p.get("content").and_then(|c| c.as_str())));
    let text = text.filter(|s| !s.is_empty())?;
    Some(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Given event JSON with matching session_id and properties.text,
    /// When extract_text_delta is called,
    /// Then returns the text.
    #[test]
    fn extract_text_delta_matching_session_properties_text() {
        let v = serde_json::json!({
            "sessionID": "ses_123",
            "properties": { "text": "hello" }
        });
        assert_eq!(extract_text_delta(&v, "ses_123"), Some("hello".to_string()));
    }

    /// Given event JSON with non-matching session_id,
    /// When extract_text_delta is called,
    /// Then returns None.
    #[test]
    fn extract_text_delta_non_matching_session_returns_none() {
        let v = serde_json::json!({
            "sessionID": "ses_other",
            "properties": { "text": "hello" }
        });
        assert_eq!(extract_text_delta(&v, "ses_123"), None);
    }

    /// Given event JSON with properties.content as text source,
    /// When extract_text_delta is called,
    /// Then returns the content.
    #[test]
    fn extract_text_delta_properties_content() {
        let v = serde_json::json!({
            "sessionId": "ses_1",
            "properties": { "content": "world" }
        });
        assert_eq!(extract_text_delta(&v, "ses_1"), Some("world".to_string()));
    }

    /// Given event JSON with empty text,
    /// When extract_text_delta is called,
    /// Then returns None.
    #[test]
    fn extract_text_delta_empty_text_returns_none() {
        let v = serde_json::json!({
            "sessionID": "ses_1",
            "properties": { "text": "" }
        });
        assert_eq!(extract_text_delta(&v, "ses_1"), None);
    }

    /// Given OpenCode message.part.updated with properties.delta and part.sessionID,
    /// When extract_text_delta is called,
    /// Then returns the delta and matches session from part.
    #[test]
    fn extract_text_delta_part_delta() {
        let v = serde_json::json!({
            "type": "message.part.updated",
            "properties": {
                "delta": "hello ",
                "part": {
                    "sessionID": "ses_1",
                    "messageID": "msg_1",
                    "type": "text",
                    "text": "hello "
                }
            }
        });
        assert_eq!(extract_text_delta(&v, "ses_1"), Some("hello ".to_string()));
        assert_eq!(extract_text_delta(&v, "ses_other"), None);
    }

    /// Given event JSON with type containing "finish" and matching session,
    /// When extract_completion is called,
    /// Then returns true.
    #[test]
    fn extract_completion_finish_type() {
        let v = serde_json::json!({
            "type": "message.part.updated",
            "sessionId": "ses_1",
            "properties": { "partType": "finish", "finishReason": "end_turn" }
        });
        assert!(extract_completion(&v, "ses_1"));
    }

    /// Given event JSON with non-matching session_id,
    /// When extract_completion is called,
    /// Then returns false.
    #[test]
    fn extract_completion_non_matching_session_returns_false() {
        let v = serde_json::json!({
            "type": "message.part.updated",
            "sessionId": "ses_other",
            "properties": { "partType": "finish" }
        });
        assert!(!extract_completion(&v, "ses_1"));
    }

    /// Given session.idle event with matching sessionID,
    /// When extract_completion is called,
    /// Then returns true.
    #[test]
    fn extract_completion_session_idle() {
        let v = serde_json::json!({
            "type": "session.idle",
            "properties": { "sessionID": "ses_1" }
        });
        assert!(extract_completion(&v, "ses_1"));
    }

    /// Given session.status with status.type "idle" and matching sessionID,
    /// When extract_completion is called,
    /// Then returns true.
    #[test]
    fn extract_completion_session_status_idle() {
        let v = serde_json::json!({
            "type": "session.status",
            "properties": { "sessionID": "ses_1", "status": { "type": "idle" } }
        });
        assert!(extract_completion(&v, "ses_1"));
    }

    /// Given message.part.updated with part.type "step-finish" and matching sessionID,
    /// When extract_completion is called,
    /// Then returns true.
    #[test]
    fn extract_completion_step_finish() {
        let v = serde_json::json!({
            "type": "message.part.updated",
            "properties": {
                "sessionID": "ses_1",
                "part": { "type": "step-finish", "reason": "stop" }
            }
        });
        assert!(extract_completion(&v, "ses_1"));
    }

    /// Given message.updated with info.finish and matching sessionID,
    /// When extract_completion is called,
    /// Then returns true.
    #[test]
    fn extract_completion_message_updated_finish() {
        let v = serde_json::json!({
            "type": "message.updated",
            "properties": {
                "info": { "sessionID": "ses_1", "finish": "stop", "role": "assistant" }
            }
        });
        assert!(extract_completion(&v, "ses_1"));
    }
}
