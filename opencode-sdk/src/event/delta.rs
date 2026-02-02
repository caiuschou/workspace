//! Text delta extraction: extract_text_delta for SSE events.

/// Extracts the latest incremental text from event JSON if it matches our session.
///
/// Each event yields at most one string: the **new incremental content** for this event.
/// Prefer `properties.delta` (streaming increment), then `properties.part.text`,
/// then `properties.text` / `properties.content`. When the server sends `delta`, the result
/// is exactly the new chunk; when it sends `part.text`/`text`/`content`, the result is
/// whatever the server put in that field (increment or cumulative, server-dependent).
///
/// For OpenCode message.part.updated see 17-event-format.md.
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
    if let Some(p) = props {
        let part_session = p
            .get("part")
            .and_then(|part| part.get("sessionID").or(part.get("sessionId")).and_then(|s| s.as_str()));
        if let Some(s) = part_session {
            if s != session_id {
                return None;
            }
        }
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
