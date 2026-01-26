//! Session management types for HTTP transport.

use serde::{Deserialize, Serialize};

/// Unique session identifier for HTTP connections.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new random session ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create a session ID from an existing string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the session ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Token for resuming a session after reconnection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumptionToken {
    /// The session ID this token belongs to.
    pub session_id: SessionId,
    /// The last SSE event ID received by the client.
    pub last_event_id: Option<String>,
    /// Unix timestamp (milliseconds) when the token was created.
    pub timestamp: u64,
}

impl ResumptionToken {
    /// Create a new resumption token.
    pub fn new(session_id: SessionId, last_event_id: Option<String>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            session_id,
            last_event_id,
            timestamp,
        }
    }

    /// Encode the token as a base64 string.
    pub fn encode(&self) -> Result<String, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(base64_encode(json.as_bytes()))
    }

    /// Decode a token from a base64 string.
    pub fn decode(encoded: &str) -> Result<Self, ResumptionTokenError> {
        let bytes = base64_decode(encoded).map_err(|_| ResumptionTokenError::InvalidBase64)?;
        let json =
            std::str::from_utf8(&bytes).map_err(|_| ResumptionTokenError::InvalidUtf8)?;
        serde_json::from_str(json).map_err(ResumptionTokenError::Json)
    }
}

/// Errors that can occur when decoding a resumption token.
#[derive(Debug, thiserror::Error)]
pub enum ResumptionTokenError {
    /// Invalid base64 encoding.
    #[error("invalid base64 encoding")]
    InvalidBase64,
    /// Invalid UTF-8 in decoded data.
    #[error("invalid UTF-8")]
    InvalidUtf8,
    /// JSON parsing failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let chunks = data.chunks(3);

    for chunk in chunks {
        let mut n: u32 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            n |= (byte as u32) << (16 - i * 8);
        }

        let indices = match chunk.len() {
            3 => vec![
                (n >> 18) & 0x3F,
                (n >> 12) & 0x3F,
                (n >> 6) & 0x3F,
                n & 0x3F,
            ],
            2 => vec![(n >> 18) & 0x3F, (n >> 12) & 0x3F, (n >> 6) & 0x3F],
            1 => vec![(n >> 18) & 0x3F, (n >> 12) & 0x3F],
            _ => vec![],
        };

        for idx in indices {
            result.push(ALPHABET[idx as usize] as char);
        }

        for _ in 0..(3 - chunk.len()) {
            result.push('=');
        }
    }

    result
}

fn base64_decode(data: &str) -> Result<Vec<u8>, ()> {
    const DECODE_TABLE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1,
        -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4,
        5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1,
        -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];

    let data = data.trim_end_matches('=');
    let mut result = Vec::with_capacity(data.len() * 3 / 4);
    let mut buffer: u32 = 0;
    let mut bits_collected: u8 = 0;

    for c in data.chars() {
        let value = if (c as usize) < 128 {
            DECODE_TABLE[c as usize]
        } else {
            -1
        };

        if value < 0 {
            return Err(());
        }

        buffer = (buffer << 6) | (value as u32);
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            result.push((buffer >> bits_collected) as u8);
            buffer &= (1 << bits_collected) - 1;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_creation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_session_id_from_string() {
        let id = SessionId::from_string("test-session-123");
        assert_eq!(id.as_str(), "test-session-123");
    }

    #[test]
    fn test_resumption_token_encode_decode() {
        let session_id = SessionId::from_string("test-session");
        let token = ResumptionToken::new(session_id.clone(), Some("event-42".to_string()));

        let encoded = token.encode().unwrap();
        let decoded = ResumptionToken::decode(&encoded).unwrap();

        assert_eq!(decoded.session_id, session_id);
        assert_eq!(decoded.last_event_id, Some("event-42".to_string()));
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = b"Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
