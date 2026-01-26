//! HTTP request handler for MCP server.

use std::sync::Arc;

use mcp_core::stdio::{deserialize_message, serialize_message, JsonRpcMessage};

use crate::server::McpServer;

use super::error::HttpServerError;
use super::session_manager::{SessionConfig, SessionManager, SessionState};
use super::sse_writer::{SseResponseBuilder, SseWriter};

/// Configuration for the HTTP server handler.
#[derive(Debug, Clone)]
pub struct HttpServerOptions {
    /// Session configuration.
    pub session_config: SessionConfig,
    /// Whether SSE streaming is enabled.
    pub enable_sse: bool,
    /// Whether single JSON responses are enabled.
    pub enable_single_response: bool,
    /// Base URL for the server (used in endpoint events).
    pub base_url: Option<String>,
    /// Endpoint path.
    pub endpoint_path: String,
}

impl Default for HttpServerOptions {
    fn default() -> Self {
        Self {
            session_config: SessionConfig::default(),
            enable_sse: true,
            enable_single_response: true,
            base_url: None,
            endpoint_path: "/mcp".to_string(),
        }
    }
}

/// Result of handling an HTTP request.
pub enum HttpResponse {
    /// JSON response.
    Json {
        status: u16,
        body: String,
        session_id: Option<String>,
    },
    /// SSE stream response.
    Sse {
        session_id: String,
        /// Function to write events to the SSE stream.
        /// Takes a writer and returns when the stream should close.
        writer_fn: Box<dyn FnOnce(Box<dyn std::io::Write + Send>) + Send>,
    },
    /// Empty response (e.g., for DELETE).
    Empty { status: u16 },
    /// Error response.
    Error { status: u16, message: String },
}

/// HTTP request handler for MCP.
///
/// This handler processes HTTP requests and delegates to the MCP server.
/// It's designed to be framework-agnostic and can be integrated with
/// any HTTP server (axum, actix-web, hyper, etc.).
pub struct HttpServerHandler {
    server: Arc<McpServer>,
    session_manager: Arc<SessionManager>,
    options: HttpServerOptions,
}

impl HttpServerHandler {
    /// Create a new HTTP server handler.
    pub fn new(server: Arc<McpServer>, options: HttpServerOptions) -> Self {
        let session_manager = Arc::new(SessionManager::new(options.session_config.clone()));
        Self {
            server,
            session_manager,
            options,
        }
    }

    /// Create a new handler with default options.
    pub fn with_server(server: Arc<McpServer>) -> Self {
        Self::new(server, HttpServerOptions::default())
    }

    /// Get the session manager.
    pub fn session_manager(&self) -> &Arc<SessionManager> {
        &self.session_manager
    }

    /// Handle a POST request (send message).
    pub fn handle_post(
        &self,
        session_id_header: Option<&str>,
        content_type: Option<&str>,
        body: &[u8],
    ) -> HttpResponse {
        // Validate content type
        if let Some(ct) = content_type {
            if !ct.starts_with("application/json") {
                return HttpResponse::Error {
                    status: 415,
                    message: format!("Unsupported content type: {}", ct),
                };
            }
        }

        // Parse the JSON-RPC message
        let body_str = match std::str::from_utf8(body) {
            Ok(s) => s,
            Err(e) => {
                return HttpResponse::Error {
                    status: 400,
                    message: format!("Invalid UTF-8: {}", e),
                };
            }
        };

        let message = match deserialize_message(body_str) {
            Ok(m) => m,
            Err(e) => {
                return HttpResponse::Error {
                    status: 400,
                    message: format!("Invalid JSON-RPC message: {}", e),
                };
            }
        };

        // Get or create session
        let (session, is_new) = match self.get_or_create_session(session_id_header) {
            Ok(result) => result,
            Err(e) => {
                return HttpResponse::Error {
                    status: e.status_code(),
                    message: e.to_string(),
                };
            }
        };

        let session_id = session.session_id.to_string();

        // Handle the message
        match message {
            JsonRpcMessage::Request(request) => {
                // Process request synchronously
                let result = futures::executor::block_on(
                    self.server.server().handle_request(request, Some(session_id.clone())),
                );

                match result {
                    Ok(response) => {
                        let response_msg = JsonRpcMessage::Result(response);
                        match serialize_message(&response_msg) {
                            Ok(body) => HttpResponse::Json {
                                status: 200,
                                body,
                                session_id: if is_new { Some(session_id) } else { None },
                            },
                            Err(e) => HttpResponse::Error {
                                status: 500,
                                message: format!("Serialization error: {}", e),
                            },
                        }
                    }
                    Err(e) => HttpResponse::Error {
                        status: 500,
                        message: format!("Server error: {}", e),
                    },
                }
            }
            JsonRpcMessage::Notification(notification) => {
                // Process notification (no response expected)
                let _ = futures::executor::block_on(
                    self.server
                        .server()
                        .handle_notification(notification, Some(session_id.clone())),
                );

                HttpResponse::Empty { status: 202 }
            }
            JsonRpcMessage::Result(_) => {
                // Clients shouldn't send results
                HttpResponse::Error {
                    status: 400,
                    message: "Unexpected result message from client".to_string(),
                }
            }
        }
    }

    /// Handle a GET request (establish SSE connection).
    pub fn handle_get(
        &self,
        session_id_header: Option<&str>,
        _last_event_id: Option<&str>,
        accept: Option<&str>,
    ) -> HttpResponse {
        // Check if SSE is enabled
        if !self.options.enable_sse {
            return HttpResponse::Error {
                status: 405,
                message: "SSE not enabled".to_string(),
            };
        }

        // Validate accept header
        if let Some(accept) = accept {
            if !accept.contains("text/event-stream") {
                return HttpResponse::Error {
                    status: 406,
                    message: "Must accept text/event-stream".to_string(),
                };
            }
        }

        // Get or create session
        let (session, _is_new) = match self.get_or_create_session(session_id_header) {
            Ok(result) => result,
            Err(e) => {
                return HttpResponse::Error {
                    status: e.status_code(),
                    message: e.to_string(),
                };
            }
        };

        let session_id = session.session_id.to_string();
        let _session_manager = Arc::clone(&self.session_manager);
        let endpoint_url = self.endpoint_url();

        // Return SSE response
        HttpResponse::Sse {
            session_id: session_id.clone(),
            writer_fn: Box::new(move |mut writer: Box<dyn std::io::Write + Send>| {
                let mut sse_writer = SseWriter::with_session(&mut *writer, &session);

                // Build response
                let builder = SseResponseBuilder::new()
                    .send_session_ready(true)
                    .endpoint_url(endpoint_url);

                if let Err(e) = builder.initialize(&mut sse_writer, &session) {
                    eprintln!("SSE initialization error: {}", e);
                }

                // The actual message streaming would be handled by the caller
                // This is just the setup phase
            }),
        }
    }

    /// Handle a DELETE request (close session).
    pub fn handle_delete(&self, session_id_header: Option<&str>) -> HttpResponse {
        let session_id = match session_id_header {
            Some(id) => id,
            None => {
                return HttpResponse::Error {
                    status: 400,
                    message: "Missing session ID".to_string(),
                };
            }
        };

        match self.session_manager.remove_session(session_id) {
            Some(_) => HttpResponse::Empty { status: 204 },
            None => HttpResponse::Error {
                status: 404,
                message: format!("Session not found: {}", session_id),
            },
        }
    }

    /// Get or create a session based on the session ID header.
    fn get_or_create_session(
        &self,
        session_id_header: Option<&str>,
    ) -> Result<(SessionState, bool), HttpServerError> {
        match session_id_header {
            Some(id) => {
                // Try to get existing session
                match self.session_manager.touch_session(id) {
                    Some(session) => Ok((session, false)),
                    None => {
                        // Session not found, create new one
                        let session = self.session_manager.create_session()?;
                        Ok((session, true))
                    }
                }
            }
            None => {
                // No session ID, create new one
                let session = self.session_manager.create_session()?;
                Ok((session, true))
            }
        }
    }

    /// Get the full endpoint URL.
    fn endpoint_url(&self) -> String {
        match &self.options.base_url {
            Some(base) => format!("{}{}", base.trim_end_matches('/'), self.options.endpoint_path),
            None => self.options.endpoint_path.clone(),
        }
    }

    /// Clean up expired sessions.
    pub fn cleanup_sessions(&self) -> usize {
        self.session_manager.cleanup_expired()
    }
}

/// Extract headers from a request in a framework-agnostic way.
pub struct RequestHeaders<'a> {
    pub session_id: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub accept: Option<&'a str>,
    pub last_event_id: Option<&'a str>,
    pub resumption_token: Option<&'a str>,
}

impl<'a> RequestHeaders<'a> {
    /// Create empty headers.
    pub fn empty() -> Self {
        Self {
            session_id: None,
            content_type: None,
            accept: None,
            last_event_id: None,
            resumption_token: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{ServerOptions, McpServer};
    use mcp_core::types::{BaseMetadata, Icons, Implementation, RequestMessage};

    fn create_test_handler() -> HttpServerHandler {
        let server_info = Implementation {
            base: BaseMetadata {
                name: "test-server".to_string(),
                title: None,
            },
            icons: Icons::default(),
            version: "0.1.0".to_string(),
            website_url: None,
            description: None,
        };
        let server = McpServer::new(server_info, ServerOptions::default());
        HttpServerHandler::with_server(Arc::new(server))
    }

    #[test]
    fn test_handle_post_invalid_json() {
        let handler = create_test_handler();

        let response = handler.handle_post(None, Some("application/json"), b"not json");

        match response {
            HttpResponse::Error { status, message } => {
                assert_eq!(status, 400);
                assert!(message.contains("Invalid JSON-RPC"));
            }
            _ => panic!("Expected error response"),
        }
    }

    #[test]
    fn test_handle_post_wrong_content_type() {
        let handler = create_test_handler();

        let response = handler.handle_post(None, Some("text/plain"), b"{}");

        match response {
            HttpResponse::Error { status, .. } => {
                assert_eq!(status, 415);
            }
            _ => panic!("Expected error response"),
        }
    }

    #[test]
    fn test_handle_delete_missing_session() {
        let handler = create_test_handler();

        let response = handler.handle_delete(None);

        match response {
            HttpResponse::Error { status, .. } => {
                assert_eq!(status, 400);
            }
            _ => panic!("Expected error response"),
        }
    }

    #[test]
    fn test_handle_delete_nonexistent_session() {
        let handler = create_test_handler();

        let response = handler.handle_delete(Some("nonexistent"));

        match response {
            HttpResponse::Error { status, .. } => {
                assert_eq!(status, 404);
            }
            _ => panic!("Expected error response"),
        }
    }

    #[test]
    fn test_session_creation() {
        let handler = create_test_handler();

        // First request should create a session
        let request = RequestMessage::new(
            mcp_core::types::MessageId::Number(1),
            "ping",
            serde_json::json!({}),
        );
        let body = serde_json::to_vec(&request).unwrap();

        let response = handler.handle_post(None, Some("application/json"), &body);

        // Should get a response (might be an error since "ping" isn't registered,
        // but session should still be created)
        assert!(handler.session_manager.session_count() > 0);
    }
}
