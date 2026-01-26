//! Legacy SSE transport for backwards compatibility.
//!
//! This module implements the deprecated HTTP+SSE transport (protocol version 2024-11-05)
//! for backwards compatibility with older MCP clients.
//!
//! ## Differences from Streamable HTTP
//!
//! | Feature | Legacy SSE | Streamable HTTP |
//! |---------|-----------|-----------------|
//! | Session ID | URL query param `?sessionId=xxx` | `Mcp-Session-Id` header |
//! | Endpoint | Sends `endpoint` event with POST URL | Not needed |
//! | POST response | Returns 202 Accepted (no content) | Returns JSON or SSE stream |
//! | Reconnection | No Last-Event-ID support | Supports Last-Event-ID replay |
//!
//! ## Usage
//!
//! This transport is deprecated and should only be used for backwards compatibility
//! with older clients. New implementations should use Streamable HTTP.

use std::collections::HashMap;
use std::sync::Arc;

use mcp_core::stdio::{deserialize_message, serialize_message, JsonRpcMessage};

use super::error::HttpServerError;
use crate::server::McpServer;

/// Configuration for legacy SSE transport.
#[derive(Debug, Clone)]
pub struct LegacySseConfig {
    /// Base endpoint path (default: "/sse")
    pub endpoint_path: String,
    /// POST endpoint path for receiving messages (default: "/message")
    pub message_path: String,
}

impl Default for LegacySseConfig {
    fn default() -> Self {
        Self {
            endpoint_path: "/sse".to_string(),
            message_path: "/message".to_string(),
        }
    }
}

/// Session state for legacy SSE transport.
pub struct LegacySseSession {
    /// Session ID
    #[allow(dead_code)]
    pub session_id: String,
    /// Sender for SSE messages
    #[cfg(feature = "tokio")]
    pub tx: tokio::sync::mpsc::Sender<JsonRpcMessage>,
}

/// State manager for legacy SSE transport.
pub struct LegacySseState {
    server: Arc<McpServer>,
    config: LegacySseConfig,
    #[cfg(feature = "tokio")]
    sessions: tokio::sync::RwLock<HashMap<String, LegacySseSession>>,
}

impl LegacySseState {
    /// Create a new legacy SSE state.
    pub fn new(server: Arc<McpServer>, config: LegacySseConfig) -> Self {
        Self {
            server,
            config,
            #[cfg(feature = "tokio")]
            sessions: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Get the MCP server.
    pub fn server(&self) -> &Arc<McpServer> {
        &self.server
    }

    /// Get the configuration.
    pub fn config(&self) -> &LegacySseConfig {
        &self.config
    }

    /// Get the full POST endpoint URL with session ID.
    pub fn message_endpoint(&self, session_id: &str) -> String {
        format!("{}?sessionId={}", self.config.message_path, session_id)
    }

    /// Register a new session.
    #[cfg(feature = "tokio")]
    pub async fn register_session(
        &self,
        session_id: String,
        tx: tokio::sync::mpsc::Sender<JsonRpcMessage>,
    ) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(
            session_id.clone(),
            LegacySseSession { session_id, tx },
        );
    }

    /// Unregister a session.
    #[cfg(feature = "tokio")]
    pub async fn unregister_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }

    /// Get a session by ID.
    #[cfg(feature = "tokio")]
    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> Option<tokio::sync::mpsc::Sender<JsonRpcMessage>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.tx.clone())
    }

    /// Send a message to a session.
    #[cfg(feature = "tokio")]
    pub async fn send_to_session(
        &self,
        session_id: &str,
        message: JsonRpcMessage,
    ) -> Result<(), HttpServerError> {
        let tx = self
            .get_session(session_id)
            .await
            .ok_or_else(|| HttpServerError::SessionNotFound(session_id.to_string()))?;

        tx.send(message)
            .await
            .map_err(|_| HttpServerError::SessionNotFound(session_id.to_string()))
    }

    /// Get the number of active sessions.
    #[cfg(feature = "tokio")]
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

/// Generate a unique session ID.
pub fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

// ============================================================================
// Axum integration (feature = "axum")
// ============================================================================

#[cfg(feature = "axum")]
pub mod axum_legacy {
    use super::*;

    use std::convert::Infallible;

    use axum::body::Body;
    use axum::extract::{Query, State};
    use axum::http::{header, Method, StatusCode};
    use axum::response::sse::{Event, KeepAlive, Sse};
    use axum::response::{IntoResponse, Response};
    use axum::routing::{get, post};
    use axum::Router;
    use futures::stream::Stream;
    use tower_http::cors::{Any, CorsLayer};

    /// Query parameters for legacy SSE.
    #[derive(Debug, serde::Deserialize)]
    pub struct SessionQuery {
        #[serde(rename = "sessionId")]
        pub session_id: Option<String>,
    }

    /// Create an axum router for legacy SSE transport.
    pub fn create_legacy_sse_router(state: Arc<LegacySseState>) -> Router {
        Router::new()
            .route(&state.config.endpoint_path, get(handle_sse))
            .route(&state.config.message_path, post(handle_post))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST])
                    .allow_headers([header::CONTENT_TYPE, header::ACCEPT]),
            )
            .with_state(state)
    }

    /// Handle GET request to establish SSE connection.
    async fn handle_sse(State(state): State<Arc<LegacySseState>>) -> Response {
        // Generate session ID
        let session_id = generate_session_id();

        // Create channel for messages
        let (tx, rx) = tokio::sync::mpsc::channel::<JsonRpcMessage>(100);

        // Register session
        state.register_session(session_id.clone(), tx).await;

        // Create SSE stream
        let endpoint = state.message_endpoint(&session_id);
        let stream = create_legacy_sse_stream(session_id.clone(), endpoint, rx, state.clone());

        Sse::new(stream)
            .keep_alive(KeepAlive::default())
            .into_response()
    }

    /// Handle POST request with message.
    async fn handle_post(
        State(state): State<Arc<LegacySseState>>,
        Query(query): Query<SessionQuery>,
        body: String,
    ) -> Response {
        // Validate session ID
        let session_id = match query.session_id {
            Some(id) => id,
            None => {
                return error_response(StatusCode::BAD_REQUEST, "Missing sessionId parameter");
            }
        };

        // Check if session exists
        let tx = match state.get_session(&session_id).await {
            Some(tx) => tx,
            None => {
                return error_response(
                    StatusCode::NOT_FOUND,
                    &format!("Session not found: {}", session_id),
                );
            }
        };

        // Parse message
        let message = match deserialize_message(&body) {
            Ok(m) => m,
            Err(e) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid JSON-RPC message: {}", e),
                );
            }
        };

        // Handle message
        match message {
            JsonRpcMessage::Request(request) => {
                let result = state
                    .server
                    .server()
                    .handle_request(request, Some(session_id.clone()))
                    .await;

                match result {
                    Ok(response) => {
                        // Send response via SSE
                        let response_msg = JsonRpcMessage::Result(response);
                        let _ = tx.send(response_msg).await;
                    }
                    Err(e) => {
                        eprintln!("Server error: {}", e);
                    }
                }
            }
            JsonRpcMessage::Notification(notification) => {
                let _ = state
                    .server
                    .server()
                    .handle_notification(notification, Some(session_id.clone()))
                    .await;
            }
            JsonRpcMessage::Result(_) => {
                // Ignore unexpected result from client
            }
        }

        // Return 202 Accepted (legacy behavior)
        Response::builder()
            .status(StatusCode::ACCEPTED)
            .body(Body::from("Accepted"))
            .unwrap()
    }

    /// Create legacy SSE stream.
    fn create_legacy_sse_stream(
        session_id: String,
        endpoint: String,
        mut rx: tokio::sync::mpsc::Receiver<JsonRpcMessage>,
        state: Arc<LegacySseState>,
    ) -> impl Stream<Item = Result<Event, Infallible>> {
        async_stream::stream! {
            // Send endpoint event (legacy protocol requirement)
            yield Ok(Event::default().event("endpoint").data(&endpoint));

            // Stream messages
            while let Some(message) = rx.recv().await {
                if let Ok(json) = serialize_message(&message) {
                    yield Ok(Event::default().event("message").data(json));
                }
            }

            // Cleanup
            state.unregister_session(&session_id).await;
        }
    }

    /// Create a JSON error response.
    fn error_response(status: StatusCode, message: &str) -> Response {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "error": {
                "code": -32000,
                "message": message
            }
        });

        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }
}

#[cfg(feature = "axum")]
pub use axum_legacy::create_legacy_sse_router;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LegacySseConfig::default();
        assert_eq!(config.endpoint_path, "/sse");
        assert_eq!(config.message_path, "/message");
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_message_endpoint() {
        let server = Arc::new(crate::server::McpServer::new(
            mcp_core::types::Implementation {
                base: mcp_core::types::BaseMetadata {
                    name: "test".to_string(),
                    title: None,
                },
                icons: mcp_core::types::Icons::default(),
                version: "0.1.0".to_string(),
                website_url: None,
                description: None,
            },
            crate::server::ServerOptions::default(),
        ));
        let state = LegacySseState::new(server, LegacySseConfig::default());
        let endpoint = state.message_endpoint("abc123");
        assert_eq!(endpoint, "/message?sessionId=abc123");
    }
}
