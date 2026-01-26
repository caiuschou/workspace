//! SSE response writer for HTTP server.

use std::io::Write;

use mcp_core::http::SseEvent;
use mcp_core::stdio::JsonRpcMessage;

use super::error::HttpServerError;
use super::session_manager::SessionState;

/// Writer for Server-Sent Events responses.
pub struct SseWriter<W: Write> {
    writer: W,
    event_counter: u64,
    session_id: Option<String>,
}

impl<W: Write> SseWriter<W> {
    /// Create a new SSE writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            event_counter: 0,
            session_id: None,
        }
    }

    /// Create a new SSE writer with session context.
    pub fn with_session(writer: W, session: &SessionState) -> Self {
        Self {
            writer,
            event_counter: session.event_counter,
            session_id: Some(session.session_id.to_string()),
        }
    }

    /// Generate the next event ID.
    fn next_event_id(&mut self) -> String {
        self.event_counter += 1;
        match &self.session_id {
            Some(sid) => format!("{}-{}", sid, self.event_counter),
            None => format!("{}", self.event_counter),
        }
    }

    /// Write a raw SSE event.
    pub fn write_event(&mut self, event: &SseEvent) -> Result<(), HttpServerError> {
        let sse_string = event.to_sse_string();
        self.writer
            .write_all(sse_string.as_bytes())
            .map_err(|e| HttpServerError::Io(e.to_string()))?;
        self.writer
            .flush()
            .map_err(|e| HttpServerError::Io(e.to_string()))?;
        Ok(())
    }

    /// Write a JSON-RPC message as an SSE event.
    pub fn write_message(&mut self, message: &JsonRpcMessage) -> Result<String, HttpServerError> {
        let event_id = self.next_event_id();
        let event = SseEvent::Message {
            id: Some(event_id.clone()),
            data: message.clone(),
        };
        self.write_event(&event)?;
        Ok(event_id)
    }

    /// Write a ping event.
    pub fn write_ping(&mut self) -> Result<(), HttpServerError> {
        self.write_event(&SseEvent::Ping)
    }

    /// Write an endpoint event.
    pub fn write_endpoint(&mut self, url: &str) -> Result<(), HttpServerError> {
        self.write_event(&SseEvent::Endpoint {
            endpoint_url: url.to_string(),
        })
    }

    /// Write a session ready event.
    pub fn write_session_ready(
        &mut self,
        session_id: &mcp_core::http::SessionId,
    ) -> Result<(), HttpServerError> {
        self.write_event(&SseEvent::SessionReady {
            session_id: session_id.clone(),
        })
    }

    /// Get the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Get the current event counter.
    pub fn event_counter(&self) -> u64 {
        self.event_counter
    }
}

/// Builder for SSE responses.
#[derive(Debug, Clone)]
pub struct SseResponseBuilder {
    /// Whether to include the session ID in the response.
    pub include_session_id: bool,
    /// Whether to send an initial session ready event.
    pub send_session_ready: bool,
    /// Whether to send an endpoint event.
    pub send_endpoint: bool,
    /// The endpoint URL to send.
    pub endpoint_url: Option<String>,
    /// Keep-alive interval in seconds.
    pub keep_alive_interval: Option<u64>,
}

impl Default for SseResponseBuilder {
    fn default() -> Self {
        Self {
            include_session_id: true,
            send_session_ready: true,
            send_endpoint: false,
            endpoint_url: None,
            keep_alive_interval: Some(30),
        }
    }
}

impl SseResponseBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to include the session ID.
    pub fn include_session_id(mut self, include: bool) -> Self {
        self.include_session_id = include;
        self
    }

    /// Set whether to send a session ready event.
    pub fn send_session_ready(mut self, send: bool) -> Self {
        self.send_session_ready = send;
        self
    }

    /// Set the endpoint URL to send.
    pub fn endpoint_url(mut self, url: impl Into<String>) -> Self {
        self.send_endpoint = true;
        self.endpoint_url = Some(url.into());
        self
    }

    /// Set the keep-alive interval.
    pub fn keep_alive_interval(mut self, seconds: Option<u64>) -> Self {
        self.keep_alive_interval = seconds;
        self
    }

    /// Initialize an SSE writer with the configured options.
    pub fn initialize<W: Write>(
        &self,
        writer: &mut SseWriter<W>,
        session: &SessionState,
    ) -> Result<(), HttpServerError> {
        // Send session ready event
        if self.send_session_ready {
            writer.write_session_ready(&session.session_id)?;
        }

        // Send endpoint event
        if self.send_endpoint {
            if let Some(ref url) = self.endpoint_url {
                writer.write_endpoint(url)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_core::http::SessionId;
    use mcp_core::types::ResultMessage;

    #[test]
    fn test_sse_writer_message() {
        let mut buffer = Vec::new();
        let mut writer = SseWriter::new(&mut buffer);

        let message = JsonRpcMessage::Result(ResultMessage::success(
            mcp_core::types::MessageId::Number(1),
            serde_json::json!({"test": true}),
        ));

        let event_id = writer.write_message(&message).unwrap();
        assert_eq!(event_id, "1");

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("id: 1"));
        assert!(output.contains("event: message"));
        assert!(output.contains("data:"));
    }

    #[test]
    fn test_sse_writer_ping() {
        let mut buffer = Vec::new();
        let mut writer = SseWriter::new(&mut buffer);

        writer.write_ping().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains(":ping"));
    }

    #[test]
    fn test_sse_writer_with_session() {
        let session_id = SessionId::from_string("test-session");
        let session = SessionState {
            session_id: session_id.clone(),
            created_at: std::time::Instant::now(),
            last_activity: std::time::Instant::now(),
            initialized: true,
            event_counter: 10,
            data: std::collections::HashMap::new(),
        };

        let mut buffer = Vec::new();
        let mut writer = SseWriter::with_session(&mut buffer, &session);

        let message = JsonRpcMessage::Result(ResultMessage::success(
            mcp_core::types::MessageId::Number(1),
            serde_json::json!(null),
        ));

        let event_id = writer.write_message(&message).unwrap();
        assert_eq!(event_id, "test-session-11");
    }
}
