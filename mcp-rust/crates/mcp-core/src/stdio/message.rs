use crate::types::{NotificationMessage, RequestMessage, ResultMessage};
use serde::{Deserialize, Serialize};

/// JSON-RPC payloads that can flow across stdio transports.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(RequestMessage),
    Notification(NotificationMessage),
    Result(ResultMessage),
}

/// Parse a JSON-RPC message string.
pub fn deserialize_message(line: &str) -> Result<JsonRpcMessage, serde_json::Error> {
    serde_json::from_str(line)
}

/// Serialize a JSON-RPC message and append newline delimiter.
pub fn serialize_message(message: &JsonRpcMessage) -> Result<String, serde_json::Error> {
    let mut serialized = serde_json::to_string(message)?;
    serialized.push('\n');
    Ok(serialized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialize_roundtrip_preserves_message() {
        let message = JsonRpcMessage::Request(RequestMessage::new(
            "1",
            "example",
            json!({ "data": "rust" }),
        ));
        let line = serialize_message(&message).expect("serialize should work");
        assert!(line.ends_with('\n'));
        let parsed = deserialize_message(line.trim_end_matches('\n')).expect("should parse");
        assert_eq!(parsed, message);
    }
}
