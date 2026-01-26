//! Server-Sent Events (SSE) types and parsing.

use crate::stdio::JsonRpcMessage;

use super::session::SessionId;

/// SSE event types for MCP HTTP communication.
#[derive(Debug, Clone)]
pub enum SseEvent {
    /// A JSON-RPC message event.
    Message {
        /// Optional event ID for resumption.
        id: Option<String>,
        /// The JSON-RPC message payload.
        data: JsonRpcMessage,
    },
    /// Server endpoint information (sent on connection).
    Endpoint {
        /// The URL for sending POST requests.
        endpoint_url: String,
    },
    /// Keep-alive ping event.
    Ping,
    /// Session established notification.
    SessionReady {
        /// The assigned session ID.
        session_id: SessionId,
    },
}

impl SseEvent {
    /// Serialize the event to SSE wire format.
    pub fn to_sse_string(&self) -> String {
        match self {
            SseEvent::Message { id, data } => {
                let json = serde_json::to_string(data).unwrap_or_default();
                let mut result = String::new();
                if let Some(id) = id {
                    result.push_str(&format!("id: {}\n", id));
                }
                result.push_str("event: message\n");
                result.push_str(&format!("data: {}\n\n", json));
                result
            }
            SseEvent::Endpoint { endpoint_url } => {
                format!("event: endpoint\ndata: {}\n\n", endpoint_url)
            }
            SseEvent::Ping => ":ping\n\n".to_string(),
            SseEvent::SessionReady { session_id } => {
                format!("event: session\ndata: {}\n\n", session_id.as_str())
            }
        }
    }
}

/// Parser for SSE event streams.
#[derive(Debug, Default)]
pub struct SseParser {
    buffer: String,
    current_event: Option<String>,
    current_data: Vec<String>,
    current_id: Option<String>,
}

/// A parsed SSE event with raw fields.
#[derive(Debug, Clone)]
pub struct ParsedSseEvent {
    /// The event type (e.g., "message", "endpoint").
    pub event: Option<String>,
    /// The data lines joined together.
    pub data: String,
    /// The event ID if present.
    pub id: Option<String>,
}

impl SseParser {
    /// Create a new SSE parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append data to the parser buffer.
    pub fn append(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);
    }

    /// Try to parse the next complete event from the buffer.
    pub fn next_event(&mut self) -> Option<ParsedSseEvent> {
        loop {
            if let Some(pos) = self.buffer.find('\n') {
                let line = self.buffer[..pos].to_string();
                self.buffer.drain(..=pos);

                // Remove optional \r
                let line = line.trim_end_matches('\r');

                if line.is_empty() {
                    // Empty line = dispatch event
                    if !self.current_data.is_empty() || self.current_event.is_some() {
                        let event = ParsedSseEvent {
                            event: self.current_event.take(),
                            data: self.current_data.join("\n"),
                            id: self.current_id.take(),
                        };
                        self.current_data.clear();
                        return Some(event);
                    }
                } else if let Some(stripped) = line.strip_prefix(':') {
                    // Comment line, ignore (but could be a ping)
                    if stripped.trim() == "ping" {
                        return Some(ParsedSseEvent {
                            event: Some("ping".to_string()),
                            data: String::new(),
                            id: None,
                        });
                    }
                } else if let Some(colon_pos) = line.find(':') {
                    let field = &line[..colon_pos];
                    let value = line[colon_pos + 1..].trim_start_matches(' ');

                    match field {
                        "event" => self.current_event = Some(value.to_string()),
                        "data" => self.current_data.push(value.to_string()),
                        "id" => self.current_id = Some(value.to_string()),
                        "retry" => {} // Ignore retry field for now
                        _ => {}       // Unknown field
                    }
                } else {
                    // Field with no value
                    match line {
                        "event" => self.current_event = Some(String::new()),
                        "data" => self.current_data.push(String::new()),
                        "id" => self.current_id = Some(String::new()),
                        _ => {}
                    }
                }
            } else {
                return None;
            }
        }
    }

    /// Clear the parser state.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.current_event = None;
        self.current_data.clear();
        self.current_id = None;
    }
}

impl ParsedSseEvent {
    /// Try to convert this event to an MCP SSE event.
    pub fn to_mcp_event(&self) -> Result<SseEvent, SseEventParseError> {
        match self.event.as_deref() {
            Some("message") | None => {
                let message: JsonRpcMessage =
                    serde_json::from_str(&self.data).map_err(SseEventParseError::Json)?;
                Ok(SseEvent::Message {
                    id: self.id.clone(),
                    data: message,
                })
            }
            Some("endpoint") => Ok(SseEvent::Endpoint {
                endpoint_url: self.data.clone(),
            }),
            Some("ping") => Ok(SseEvent::Ping),
            Some("session") => Ok(SseEvent::SessionReady {
                session_id: SessionId::from_string(&self.data),
            }),
            Some(other) => Err(SseEventParseError::UnknownEvent(other.to_string())),
        }
    }
}

/// Errors that can occur when parsing SSE events.
#[derive(Debug, thiserror::Error)]
pub enum SseEventParseError {
    /// JSON parsing failed.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    /// Unknown event type.
    #[error("unknown event type: {0}")]
    UnknownEvent(String),
}

/// SSE response headers.
#[derive(Debug, Clone)]
pub struct SseHeaders {
    /// Session ID header value.
    pub session_id: Option<String>,
}

impl SseHeaders {
    /// Create headers for a new session.
    pub fn new_session(session_id: &SessionId) -> Self {
        Self {
            session_id: Some(session_id.to_string()),
        }
    }
}

/// HTTP header names used in MCP HTTP transport.
pub mod headers {
    /// Session ID header.
    pub const MCP_SESSION_ID: &str = "Mcp-Session-Id";
    /// Resumption token header.
    pub const MCP_RESUMPTION_TOKEN: &str = "Mcp-Resumption-Token";
    /// Standard SSE last event ID header.
    pub const LAST_EVENT_ID: &str = "Last-Event-ID";
    /// Accept header value for SSE.
    pub const ACCEPT_SSE: &str = "text/event-stream";
    /// Content-Type header value for SSE.
    pub const CONTENT_TYPE_SSE: &str = "text/event-stream";
    /// Content-Type header value for JSON.
    pub const CONTENT_TYPE_JSON: &str = "application/json";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_to_string() {
        let event = SseEvent::Ping;
        assert_eq!(event.to_sse_string(), ":ping\n\n");

        let event = SseEvent::SessionReady {
            session_id: SessionId::from_string("test-123"),
        };
        assert_eq!(event.to_sse_string(), "event: session\ndata: test-123\n\n");

        let event = SseEvent::Endpoint {
            endpoint_url: "http://localhost:8080/mcp".to_string(),
        };
        assert_eq!(
            event.to_sse_string(),
            "event: endpoint\ndata: http://localhost:8080/mcp\n\n"
        );
    }

    #[test]
    fn test_sse_parser_simple_event() {
        let mut parser = SseParser::new();
        parser.append("event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}\n\n");

        let event = parser.next_event().unwrap();
        assert_eq!(event.event, Some("message".to_string()));
        assert_eq!(
            event.data,
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}"
        );
    }

    #[test]
    fn test_sse_parser_with_id() {
        let mut parser = SseParser::new();
        parser.append("id: 42\nevent: message\ndata: test\n\n");

        let event = parser.next_event().unwrap();
        assert_eq!(event.id, Some("42".to_string()));
        assert_eq!(event.event, Some("message".to_string()));
        assert_eq!(event.data, "test");
    }

    #[test]
    fn test_sse_parser_multiline_data() {
        let mut parser = SseParser::new();
        parser.append("data: line1\ndata: line2\ndata: line3\n\n");

        let event = parser.next_event().unwrap();
        assert_eq!(event.data, "line1\nline2\nline3");
    }

    #[test]
    fn test_sse_parser_ping() {
        let mut parser = SseParser::new();
        parser.append(":ping\n\n");

        let event = parser.next_event().unwrap();
        assert_eq!(event.event, Some("ping".to_string()));
    }

    #[test]
    fn test_sse_parser_partial_event() {
        let mut parser = SseParser::new();
        parser.append("event: message\n");

        // No complete event yet
        assert!(parser.next_event().is_none());

        parser.append("data: test\n\n");

        // Now we have a complete event
        let event = parser.next_event().unwrap();
        assert_eq!(event.event, Some("message".to_string()));
        assert_eq!(event.data, "test");
    }
}
