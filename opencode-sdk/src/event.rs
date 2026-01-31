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
                        if let Some(text) = extract_text_delta(&v, session_id) {
                            info!("stream chunk: {}", text);
                            on_text(&text);
                        } else {
                            debug!(event_type = ?v.get("type"), "event (no text delta for session)");
                        }
                    }
                }
            }
            Err(_e) => break,
        }
    }
    Ok(())
}

/// Extracts text delta from event JSON if it matches our session.
fn extract_text_delta(v: &serde_json::Value, session_id: &str) -> Option<String> {
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

    let text = v
        .get("properties")
        .and_then(|p| p.get("text").and_then(|t| t.as_str()))
        .or_else(|| v.get("text").and_then(|t| t.as_str()))
        .or_else(|| v.get("properties").and_then(|p| p.get("content").and_then(|c| c.as_str())))?;
    if text.is_empty() {
        return None;
    }
    Some(text.to_string())
}
