//! SSE stream reader for HTTP client transport.

use std::io::{BufRead, BufReader, Read};

use mcp_core::http::{SseEvent, SseParser};
use mcp_core::stdio::JsonRpcMessage;

use super::error::HttpClientError;

/// Reader for SSE event streams.
pub struct SseReader<R: Read> {
    reader: BufReader<R>,
    parser: SseParser,
    last_event_id: Option<String>,
}

impl<R: Read> SseReader<R> {
    /// Create a new SSE reader from a readable stream.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            parser: SseParser::new(),
            last_event_id: None,
        }
    }

    /// Create a new SSE reader with a custom buffer size.
    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        Self {
            reader: BufReader::with_capacity(capacity, reader),
            parser: SseParser::new(),
            last_event_id: None,
        }
    }

    /// Get the last received event ID.
    pub fn last_event_id(&self) -> Option<&str> {
        self.last_event_id.as_deref()
    }

    /// Read and parse the next SSE event.
    ///
    /// Returns `Ok(None)` if the stream has ended.
    pub fn next_event(&mut self) -> Result<Option<SseEvent>, HttpClientError> {
        loop {
            // Try to get an event from the parser buffer first
            if let Some(parsed) = self.parser.next_event() {
                // Update last event ID if present
                if let Some(ref id) = parsed.id {
                    self.last_event_id = Some(id.clone());
                }

                return parsed
                    .to_mcp_event()
                    .map(Some)
                    .map_err(|e| HttpClientError::Sse(e.to_string()));
            }

            // Read more data from the stream
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => return Ok(None), // EOF
                Ok(_) => {
                    self.parser.append(&line);
                }
                Err(e) => return Err(HttpClientError::Io(e)),
            }
        }
    }

    /// Read the next JSON-RPC message from the SSE stream.
    ///
    /// Skips non-message events (ping, endpoint, etc).
    /// Returns `Ok(None)` if the stream has ended.
    pub fn next_message(&mut self) -> Result<Option<JsonRpcMessage>, HttpClientError> {
        loop {
            match self.next_event()? {
                Some(SseEvent::Message { data, .. }) => return Ok(Some(data)),
                Some(SseEvent::Ping) => continue,
                Some(SseEvent::Endpoint { .. }) => continue,
                Some(SseEvent::SessionReady { .. }) => continue,
                None => return Ok(None),
            }
        }
    }
}

/// Async SSE reader for use with tokio.
#[cfg(feature = "tokio")]
pub mod async_reader {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader as TokioBufReader};

    /// Async reader for SSE event streams.
    #[allow(dead_code)]
    pub struct AsyncSseReader<R: AsyncRead + Unpin> {
        reader: TokioBufReader<R>,
        parser: SseParser,
        last_event_id: Option<String>,
    }

    #[allow(dead_code)]
    impl<R: AsyncRead + Unpin> AsyncSseReader<R> {
        /// Create a new async SSE reader.
        pub fn new(reader: R) -> Self {
            Self {
                reader: TokioBufReader::new(reader),
                parser: SseParser::new(),
                last_event_id: None,
            }
        }

        /// Get the last received event ID.
        pub fn last_event_id(&self) -> Option<&str> {
            self.last_event_id.as_deref()
        }

        /// Read and parse the next SSE event.
        pub async fn next_event(&mut self) -> Result<Option<SseEvent>, HttpClientError> {
            loop {
                if let Some(parsed) = self.parser.next_event() {
                    if let Some(ref id) = parsed.id {
                        self.last_event_id = Some(id.clone());
                    }

                    return parsed
                        .to_mcp_event()
                        .map(Some)
                        .map_err(|e| HttpClientError::Sse(e.to_string()));
                }

                let mut line = String::new();
                match self.reader.read_line(&mut line).await {
                    Ok(0) => return Ok(None),
                    Ok(_) => {
                        self.parser.append(&line);
                    }
                    Err(e) => return Err(HttpClientError::Io(e)),
                }
            }
        }

        /// Read the next JSON-RPC message.
        pub async fn next_message(&mut self) -> Result<Option<JsonRpcMessage>, HttpClientError> {
            loop {
                match self.next_event().await? {
                    Some(SseEvent::Message { data, .. }) => return Ok(Some(data)),
                    Some(_) => continue,
                    None => return Ok(None),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_sse_reader_message() {
        let data = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}\n\n";
        let mut reader = SseReader::new(Cursor::new(data));

        let event = reader.next_event().unwrap().unwrap();
        match event {
            SseEvent::Message { data, .. } => {
                assert!(matches!(data, JsonRpcMessage::Result(_)));
            }
            _ => panic!("expected message event"),
        }
    }

    #[test]
    fn test_sse_reader_with_id() {
        let data = "id: 42\nevent: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}\n\n";
        let mut reader = SseReader::new(Cursor::new(data));

        let event = reader.next_event().unwrap().unwrap();
        match event {
            SseEvent::Message { id, .. } => {
                assert_eq!(id, Some("42".to_string()));
            }
            _ => panic!("expected message event"),
        }

        assert_eq!(reader.last_event_id(), Some("42"));
    }

    #[test]
    fn test_sse_reader_ping() {
        let data = ":ping\n\n";
        let mut reader = SseReader::new(Cursor::new(data));

        let event = reader.next_event().unwrap().unwrap();
        assert!(matches!(event, SseEvent::Ping));
    }

    #[test]
    fn test_sse_reader_eof() {
        let data = "";
        let mut reader = SseReader::new(Cursor::new(data));

        assert!(reader.next_event().unwrap().is_none());
    }

    #[test]
    fn test_sse_reader_multiple_events() {
        let data = concat!(
            "event: session\ndata: sess-123\n\n",
            ":ping\n\n",
            "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}\n\n"
        );
        let mut reader = SseReader::new(Cursor::new(data));

        // Session event
        let event = reader.next_event().unwrap().unwrap();
        assert!(matches!(event, SseEvent::SessionReady { .. }));

        // Ping event
        let event = reader.next_event().unwrap().unwrap();
        assert!(matches!(event, SseEvent::Ping));

        // Message event
        let event = reader.next_event().unwrap().unwrap();
        assert!(matches!(event, SseEvent::Message { .. }));

        // EOF
        assert!(reader.next_event().unwrap().is_none());
    }
}
