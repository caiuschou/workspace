//! Text delta extraction: extract_text_delta for SSE events.

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
