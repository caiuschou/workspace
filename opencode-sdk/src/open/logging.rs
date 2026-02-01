//! Logging helpers for assistant reply (parts, text preview).

use crate::session::{MessageListItem, Part};
use tracing::info;

/// Max length for text/reasoning preview in logs.
const LOG_PREVIEW_LEN: usize = 500;
/// Max length for full reply text preview in logs.
const LOG_REPLY_TEXT_PREVIEW_LEN: usize = 2000;

/// Logs a single assistant reply part (tool call, reasoning, text, etc.) at info level.
pub(crate) fn log_part(part: &Part, index: usize) {
    match part.part_type.as_str() {
        "tool" | "tool_call" => {
            info!(part_index = index, tool_name = ?part.tool_name, finished = ?part.finished, "assistant part tool call");
        }
        "tool_result" => {
            info!(part_index = index, tool_name = ?part.tool_name, tool_call_id = ?part.tool_call_id, is_error = ?part.is_error, "assistant part tool result");
        }
        "reasoning" => {
            if let Some(r) = part.reasoning.as_ref().filter(|s| !s.is_empty()) {
                let preview: String = r.chars().take(LOG_PREVIEW_LEN).collect();
                let suffix = if r.len() > LOG_PREVIEW_LEN { "\n..." } else { "" };
                info!("assistant part[{}] reasoning:\n---\n{}{}\n---", index, preview, suffix);
            } else {
                info!(part_index = index, "assistant part reasoning");
            }
        }
        "image" | "image_url" => {
            info!(part_index = index, image_url = ?part.image_url, "assistant part image");
        }
        "binary" => {
            info!(part_index = index, "assistant part binary");
        }
        "finish" => {
            info!(part_index = index, finish_reason = ?part.finish_reason, "assistant part finish");
        }
        _ => {
            if let Some(t) = part.text.as_ref().filter(|s| !s.is_empty()) {
                let preview: String = t.chars().take(LOG_PREVIEW_LEN).collect();
                let suffix = if t.len() > LOG_PREVIEW_LEN { "\n..." } else { "" };
                info!("assistant part[{}] {}:\n---\n{}{}\n---", index, part.part_type, preview, suffix);
            } else {
                info!(part_index = index, part_type = %part.part_type, "assistant part (no text)");
            }
        }
    }
}

/// Logs assistant reply parts (text, tool calls, reasoning, etc.) at info level.
pub(crate) fn log_assistant_reply(reply: &MessageListItem) {
    info!(parts_count = reply.parts.len(), "assistant reply received");
    let text = reply.text_content();
    if !text.is_empty() {
        let preview: String = text.chars().take(LOG_REPLY_TEXT_PREVIEW_LEN).collect();
        let truncated = text.len() > LOG_REPLY_TEXT_PREVIEW_LEN;
        info!("assistant reply text (len={}, truncated={}):\n---\n{}\n---", text.len(), truncated, preview);
    } else {
        info!("assistant reply has no text content");
    }
    for (i, part) in reply.parts.iter().enumerate() {
        log_part(part, i);
    }
}
