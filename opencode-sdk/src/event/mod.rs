//! Event stream for real-time updates.
//!
//! Subscribes to GET /event (instance) or GET /global/event (global) for
//! streaming updates including message.part.updated with text deltas.
//!
//! Full SSE event payloads are logged at `trace` level. To inspect them,
//! set `RUST_LOG=opencode_sdk::event=trace`.

mod completion;
mod connect;
mod delta;

use crate::client::Client;
use crate::Error;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::path::Path;
use tracing::{debug, info, trace};

pub use connect::{connect_sse, SseEvent};

/// Default SSE stream timeout (seconds). Used for GET /event and GET /global/event.
const SSE_STREAM_TIMEOUT_SECS: u64 = 3600;

/// Subscribes to global event stream (GET /global/event).
///
/// Yields raw JSON event payloads. Use this for global-level events
/// (e.g. instance lifecycle) as opposed to instance-level GET /event.
pub async fn subscribe_global_events<F>(
    client: &Client,
    mut on_event: F,
) -> Result<(), Error>
where
    F: FnMut(serde_json::Value) + Send,
{
    let url = format!("{}/global/event", client.base_url());
    let response = client
        .http()
        .get(&url)
        .header("Accept", "text/event-stream")
        .timeout(std::time::Duration::from_secs(SSE_STREAM_TIMEOUT_SECS))
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
                        on_event(v);
                    }
                }
            }
            Err(e) => {
                debug!(error = %e, "global event stream error, stopping");
                break;
            }
        }
    }
    Ok(())
}

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
    let mut stream = connect_sse(client, directory).await?;
    while let Some(result) = stream.next().await {
        let ev = result?;
        if !ev.data.is_empty() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&ev.data) {
                if let Ok(payload) = serde_json::to_string_pretty(&v) {
                    trace!("event full payload:\n{}", payload);
                } else {
                    trace!("event full payload (raw): {}", ev.data);
                }
                if let Some(text) = delta::extract_text_delta(&v, session_id) {
                    info!("stream chunk: {}", text);
                    on_text(&text);
                } else {
                    debug!(event_type = ?v.get("type"), "event (no text delta for session)");
                }
            } else {
                trace!("event full payload (raw, not JSON): {}", ev.data);
            }
        }
    }
    Ok(())
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
    let mut stream = connect_sse(client, directory).await?;
    while let Some(result) = stream.next().await {
        let ev = result?;
        if !ev.data.is_empty() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&ev.data) {
                if let Ok(payload) = serde_json::to_string_pretty(&v) {
                    trace!("event full payload:\n{}", payload);
                } else {
                    trace!("event full payload (raw): {}", ev.data);
                }
                if completion::extract_completion(&v, session_id) {
                    debug!("event stream: completion event received");
                    return Ok(());
                }
                if let Some(text) = delta::extract_text_delta(&v, session_id) {
                    info!("stream chunk: {}", text);
                    on_text(&text);
                } else {
                    debug!(event_type = ?v.get("type"), "event (no text delta for session)");
                }
            } else {
                trace!("event full payload (raw, not JSON): {}", ev.data);
            }
        }
    }
    debug!("event stream ended");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::completion::extract_completion;
    use super::delta::extract_text_delta;

    #[test]
    fn extract_text_delta_matching_session_properties_text() {
        let v = serde_json::json!({
            "sessionID": "ses_123",
            "properties": { "text": "hello" }
        });
        assert_eq!(extract_text_delta(&v, "ses_123"), Some("hello".to_string()));
    }

    #[test]
    fn extract_text_delta_non_matching_session_returns_none() {
        let v = serde_json::json!({
            "sessionID": "ses_other",
            "properties": { "text": "hello" }
        });
        assert_eq!(extract_text_delta(&v, "ses_123"), None);
    }

    #[test]
    fn extract_text_delta_properties_content() {
        let v = serde_json::json!({
            "sessionId": "ses_1",
            "properties": { "content": "world" }
        });
        assert_eq!(extract_text_delta(&v, "ses_1"), Some("world".to_string()));
    }

    #[test]
    fn extract_text_delta_empty_text_returns_none() {
        let v = serde_json::json!({
            "sessionID": "ses_1",
            "properties": { "text": "" }
        });
        assert_eq!(extract_text_delta(&v, "ses_1"), None);
    }

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

    #[test]
    fn extract_completion_finish_type() {
        let v = serde_json::json!({
            "type": "message.part.updated",
            "sessionId": "ses_1",
            "properties": { "partType": "finish", "finishReason": "end_turn" }
        });
        assert!(extract_completion(&v, "ses_1"));
    }

    #[test]
    fn extract_completion_non_matching_session_returns_false() {
        let v = serde_json::json!({
            "type": "message.part.updated",
            "sessionId": "ses_other",
            "properties": { "partType": "finish" }
        });
        assert!(!extract_completion(&v, "ses_1"));
    }

    #[test]
    fn extract_completion_session_idle() {
        let v = serde_json::json!({
            "type": "session.idle",
            "properties": { "sessionID": "ses_1" }
        });
        assert!(extract_completion(&v, "ses_1"));
    }

    #[test]
    fn extract_completion_session_status_idle() {
        let v = serde_json::json!({
            "type": "session.status",
            "properties": { "sessionID": "ses_1", "status": { "type": "idle" } }
        });
        assert!(extract_completion(&v, "ses_1"));
    }

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
