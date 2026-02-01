//! Completion detection: extract_completion for SSE events.

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

    if ty == "session.idle" {
        return true;
    }
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
