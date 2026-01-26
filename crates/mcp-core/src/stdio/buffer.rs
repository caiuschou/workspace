use std::str;

use thiserror::Error;

use super::message::{JsonRpcMessage, deserialize_message};

/// Buffer that accumulates bytes from stdout until newline-delimited JSON-RPC messages appear.
#[derive(Debug, Default)]
pub struct ReadBuffer {
    buffer: Vec<u8>,
}

impl ReadBuffer {
    /// Append more bytes received from stdout to the buffer.
    pub fn append(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
    }

    /// Attempt to parse a single JSON-RPC message from the buffered bytes.
    pub fn read_message(&mut self) -> Result<Option<JsonRpcMessage>, ReadBufferError> {
        let newline = match self.buffer.iter().position(|byte| *byte == b'\n') {
            Some(index) => index,
            None => return Ok(None),
        };

        let message = {
            let line = {
                let line = str::from_utf8(&self.buffer[..newline])?;
                line.trim_end_matches('\r')
            };
            deserialize_message(line)?
        };

        self.buffer.drain(..=newline);
        Ok(Some(message))
    }

    /// Clear any buffered bytes.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Errors produced while reading JSON-RPC messages from stdout.
#[derive(Debug, Error)]
pub enum ReadBufferError {
    #[error("utf-8 error")]
    Utf8(#[from] str::Utf8Error),

    #[error("serialization failed")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NotificationMessage;
    use serde_json::json;

    #[test]
    fn read_buffer_delivers_complete_messages() {
        let mut buf = ReadBuffer::default();
        buf.append(
            b"{\"jsonrpc\":\"2.0\",\"method\":\"notify\",\"params\":{\"text\":\"hello\"}}\n",
        );
        let message = buf.read_message().expect("should parse").unwrap();
        assert_eq!(
            message,
            JsonRpcMessage::Notification(NotificationMessage::new(
                "notify",
                Some(json!({ "text": "hello" }))
            ))
        );
    }
}
