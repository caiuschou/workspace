//! Message parsing utilities.
//!
//! Parses API response bodies into MessageListItems.

use crate::Error;
use serde_json::Value;

use super::{MessageInfo, MessageListItem};

/// Parses message list from API response. Handles wrapped format {messages: [...]}.
pub(crate) fn parse_message_list(body: &str) -> Result<Vec<MessageListItem>, Error> {
    let wrapped: Value = serde_json::from_str(body)?;
    let raw: Vec<Value> = wrapped
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

/// Parses a single message item from JSON.
pub(crate) fn parse_message_item(v: &Value) -> Result<MessageListItem, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Given JSON with "messages" wrapper,
    /// When parse_message_list is called,
    /// Then returns parsed MessageListItems.
    #[test]
    fn parse_message_list_wrapped_format() {
        let body = r#"{"messages":[{"info":{"id":"msg_1","role":"user"},"parts":[{"type":"text","text":"hi"}]}]}"#;
        let items = parse_message_list(body).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].info.id, "msg_1");
        assert_eq!(items[0].info.role, "user");
        assert_eq!(items[0].parts.len(), 1);
        assert_eq!(items[0].parts[0].part_type, "text");
        assert_eq!(items[0].parts[0].text.as_deref(), Some("hi"));
    }

    /// Given JSON as top-level array,
    /// When parse_message_list is called,
    /// Then returns parsed items.
    #[test]
    fn parse_message_list_top_level_array() {
        let body = r#"[{"id":"m1","role":"assistant","parts":[{"type":"text","text":"hello"}]}]"#;
        let items = parse_message_list(body).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].info.id, "m1");
        assert_eq!(items[0].info.role, "assistant");
        assert_eq!(items[0].text_content(), "hello");
    }

    /// Given empty object or empty messages,
    /// When parse_message_list is called,
    /// Then returns empty vec.
    #[test]
    fn parse_message_list_empty() {
        assert!(parse_message_list(r#"{"messages":[]}"#).unwrap().is_empty());
        assert!(parse_message_list(r#"{}"#).unwrap().is_empty());
    }
}
