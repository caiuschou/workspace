//! Axum integration for MCP HTTP server.
//!
//! This module provides a complete axum-based HTTP handler with true SSE streaming,
//! bidirectional communication, and Last-Event-ID replay support.

#![cfg(feature = "axum")]

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::Router;
use futures::stream::Stream;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use mcp_core::http::SseEvent;
use mcp_core::stdio::{deserialize_message, serialize_message, JsonRpcMessage};

use super::broadcast::async_broadcast::SseBroadcaster;
use super::broadcast::EventBufferConfig;
use super::dns_protection::{DnsProtectionConfig, DnsProtectionLayer};
use super::error::HttpServerError;
use super::session_manager::{SessionConfig, SessionManager, SessionState};
use crate::server::McpServer;

/// Configuration for the axum HTTP handler.
#[derive(Debug, Clone)]
pub struct AxumHandlerConfig {
    /// Session configuration.
    pub session_config: SessionConfig,
    /// Event buffer configuration for Last-Event-ID replay.
    pub event_buffer_config: EventBufferConfig,
    /// Base URL for the server.
    pub base_url: Option<String>,
    /// Endpoint path (default: "/mcp").
    pub endpoint_path: String,
    /// Keep-alive interval for SSE connections.
    pub keep_alive_interval: Duration,
    /// Broadcast channel capacity per session.
    pub broadcast_capacity: usize,
    /// Enable CORS.
    pub enable_cors: bool,
    /// Enable DNS rebinding protection.
    /// When enabled, the server validates the Host header against allowed hostnames.
    pub enable_dns_rebinding_protection: bool,
    /// DNS protection configuration.
    /// If `enable_dns_rebinding_protection` is true and this is None,
    /// localhost-only protection is used by default.
    pub dns_protection_config: Option<DnsProtectionConfig>,
}

impl Default for AxumHandlerConfig {
    fn default() -> Self {
        Self {
            session_config: SessionConfig::default(),
            event_buffer_config: EventBufferConfig::default(),
            base_url: None,
            endpoint_path: "/mcp".to_string(),
            keep_alive_interval: Duration::from_secs(30),
            broadcast_capacity: 100,
            enable_cors: true,
            enable_dns_rebinding_protection: false,
            dns_protection_config: None,
        }
    }
}

/// Shared state for the axum handler.
pub struct AxumHandlerState {
    server: Arc<McpServer>,
    session_manager: SessionManager,
    broadcasters: RwLock<HashMap<String, Arc<SseBroadcaster>>>,
    config: AxumHandlerConfig,
}

impl AxumHandlerState {
    /// Create a new handler state.
    pub fn new(server: Arc<McpServer>, config: AxumHandlerConfig) -> Self {
        Self {
            server,
            session_manager: SessionManager::new(config.session_config.clone()),
            broadcasters: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Get the MCP server.
    pub fn server(&self) -> &Arc<McpServer> {
        &self.server
    }

    /// Get the session manager.
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Get or create a broadcaster for a session.
    pub async fn get_or_create_broadcaster(
        &self,
        session_id: &str,
    ) -> Arc<SseBroadcaster> {
        // Try to get existing broadcaster
        {
            let broadcasters = self.broadcasters.read().await;
            if let Some(broadcaster) = broadcasters.get(session_id) {
                return Arc::clone(broadcaster);
            }
        }

        // Create new broadcaster
        let mut broadcasters = self.broadcasters.write().await;
        // Double-check after acquiring write lock
        if let Some(broadcaster) = broadcasters.get(session_id) {
            return Arc::clone(broadcaster);
        }

        let broadcaster = Arc::new(SseBroadcaster::with_buffer_config(
            session_id.to_string(),
            self.config.broadcast_capacity,
            self.config.event_buffer_config.clone(),
        ));
        broadcasters.insert(session_id.to_string(), Arc::clone(&broadcaster));
        broadcaster
    }

    /// Remove a broadcaster for a session.
    pub async fn remove_broadcaster(&self, session_id: &str) {
        let mut broadcasters = self.broadcasters.write().await;
        broadcasters.remove(session_id);
    }

    /// Broadcast a message to a session.
    pub async fn broadcast_to_session(
        &self,
        session_id: &str,
        message: JsonRpcMessage,
    ) -> Result<String, HttpServerError> {
        let broadcaster = self.get_or_create_broadcaster(session_id).await;
        broadcaster
            .send_message(message)
            .map_err(|_| HttpServerError::SessionNotFound(session_id.to_string()))
    }

    /// Get the full endpoint URL.
    pub fn endpoint_url(&self) -> String {
        match &self.config.base_url {
            Some(base) => format!(
                "{}{}",
                base.trim_end_matches('/'),
                self.config.endpoint_path
            ),
            None => self.config.endpoint_path.clone(),
        }
    }
}

/// Create an axum router for the MCP HTTP server.
pub fn create_router(state: Arc<AxumHandlerState>) -> Router {
    let mut router = Router::new()
        .route(&state.config.endpoint_path, post(handle_post))
        .route(&state.config.endpoint_path, get(handle_get))
        .route(&state.config.endpoint_path, delete(handle_delete))
        .with_state(state.clone());

    // Apply DNS rebinding protection if enabled
    if state.config.enable_dns_rebinding_protection {
        let dns_config = state
            .config
            .dns_protection_config
            .clone()
            .unwrap_or_else(DnsProtectionConfig::localhost);
        router = router.layer(DnsProtectionLayer::new(dns_config));
    }

    if state.config.enable_cors {
        router = router.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::ACCEPT,
                    header::HeaderName::from_static("mcp-session-id"),
                    header::HeaderName::from_static("last-event-id"),
                ])
                .expose_headers([header::HeaderName::from_static("mcp-session-id")]),
        );
    }

    router
}

/// Handle POST requests (send JSON-RPC messages).
async fn handle_post(
    State(state): State<Arc<AxumHandlerState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    // Validate content type
    if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        if let Ok(ct) = content_type.to_str() {
            if !ct.starts_with("application/json") {
                return error_response(
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    &format!("Unsupported content type: {}", ct),
                );
            }
        }
    }

    // Parse the JSON-RPC message
    let message = match deserialize_message(&body) {
        Ok(m) => m,
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid JSON-RPC message: {}", e),
            );
        }
    };

    // Get or create session
    let session_id_header = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok());

    let (session, is_new) = match get_or_create_session(&state, session_id_header) {
        Ok(result) => result,
        Err(e) => {
            return error_response(StatusCode::from_u16(e.status_code()).unwrap(), &e.to_string());
        }
    };

    let session_id = session.session_id.to_string();

    // Handle the message
    match message {
        JsonRpcMessage::Request(request) => {
            let result = state
                .server
                .server()
                .handle_request(request, Some(session_id.clone()))
                .await;

            match result {
                Ok(response) => {
                    let response_msg = JsonRpcMessage::Result(response);
                    match serialize_message(&response_msg) {
                        Ok(body) => {
                            let mut response = Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/json");

                            if is_new {
                                response = response.header("mcp-session-id", &session_id);
                            }

                            response.body(Body::from(body)).unwrap()
                        }
                        Err(e) => error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &format!("Serialization error: {}", e),
                        ),
                    }
                }
                Err(e) => error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Server error: {}", e),
                ),
            }
        }
        JsonRpcMessage::Notification(notification) => {
            let _ = state
                .server
                .server()
                .handle_notification(notification, Some(session_id.clone()))
                .await;

            Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::empty())
                .unwrap()
        }
        JsonRpcMessage::Result(_) => {
            error_response(
                StatusCode::BAD_REQUEST,
                "Unexpected result message from client",
            )
        }
    }
}

/// Handle GET requests (establish SSE connection).
async fn handle_get(
    State(state): State<Arc<AxumHandlerState>>,
    headers: HeaderMap,
) -> Response {
    // Validate accept header
    if let Some(accept) = headers.get(header::ACCEPT) {
        if let Ok(accept_str) = accept.to_str() {
            if !accept_str.contains("text/event-stream") {
                return error_response(
                    StatusCode::NOT_ACCEPTABLE,
                    "Must accept text/event-stream",
                );
            }
        }
    }

    // Get or create session
    let session_id_header = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok());

    let (session, _is_new) = match get_or_create_session(&state, session_id_header) {
        Ok(result) => result,
        Err(e) => {
            return error_response(StatusCode::from_u16(e.status_code()).unwrap(), &e.to_string());
        }
    };

    let session_id = session.session_id.to_string();

    // Get Last-Event-ID for replay
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Get or create broadcaster
    let broadcaster = state.get_or_create_broadcaster(&session_id).await;

    // Create SSE stream
    let stream = create_sse_stream(
        session_id.clone(),
        broadcaster,
        last_event_id,
        state.endpoint_url(),
    );

    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(state.config.keep_alive_interval)
            .text("ping"),
    );

    let mut response = sse.into_response();
    response
        .headers_mut()
        .insert("mcp-session-id", session_id.parse().unwrap());

    response
}

/// Handle DELETE requests (close session).
async fn handle_delete(
    State(state): State<Arc<AxumHandlerState>>,
    headers: HeaderMap,
) -> Response {
    let session_id = match headers.get("mcp-session-id").and_then(|v| v.to_str().ok()) {
        Some(id) => id,
        None => {
            return error_response(StatusCode::BAD_REQUEST, "Missing session ID");
        }
    };

    // Remove session and broadcaster
    state.session_manager().remove_session(session_id);
    state.remove_broadcaster(session_id).await;

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap()
}

/// Create an SSE event stream.
fn create_sse_stream(
    session_id: String,
    broadcaster: Arc<SseBroadcaster>,
    last_event_id: Option<String>,
    endpoint_url: String,
) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        // Send session ready event
        yield Ok(Event::default()
            .event("session")
            .data(&session_id));

        // Send endpoint event
        yield Ok(Event::default()
            .event("endpoint")
            .data(&endpoint_url));

        // Replay missed events if Last-Event-ID was provided
        if let Some(ref last_id) = last_event_id {
            let replay_events = broadcaster.get_replay_events(last_id);
            for buffered in replay_events {
                if let Some(event) = sse_event_to_axum_event(&buffered.event) {
                    yield Ok(event);
                }
            }
        }

        // Subscribe to new events
        let mut rx = broadcaster.subscribe();

        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Some(axum_event) = sse_event_to_axum_event(&event) {
                        yield Ok(axum_event);
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    // Log that we missed some events
                    eprintln!("SSE stream lagged, missed {} events", n);
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    }
}

/// Convert an SseEvent to an axum Event.
fn sse_event_to_axum_event(event: &SseEvent) -> Option<Event> {
    match event {
        SseEvent::Message { id, data } => {
            let json = serde_json::to_string(data).ok()?;
            let mut e = Event::default().event("message").data(json);
            if let Some(event_id) = id {
                e = e.id(event_id.clone());
            }
            Some(e)
        }
        SseEvent::Ping => Some(Event::default().comment("ping")),
        SseEvent::SessionReady { session_id } => Some(
            Event::default()
                .event("session")
                .data(session_id.to_string()),
        ),
        SseEvent::Endpoint { endpoint_url } => Some(
            Event::default()
                .event("endpoint")
                .data(endpoint_url.clone()),
        ),
    }
}

/// Get or create a session.
fn get_or_create_session(
    state: &AxumHandlerState,
    session_id_header: Option<&str>,
) -> Result<(SessionState, bool), HttpServerError> {
    match session_id_header {
        Some(id) => match state.session_manager().touch_session(id) {
            Some(session) => Ok((session, false)),
            None => {
                let session = state.session_manager().create_session()?;
                Ok((session, true))
            }
        },
        None => {
            let session = state.session_manager().create_session()?;
            Ok((session, true))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_creation() {
        use crate::server::{McpServer, ServerOptions};
        use mcp_core::types::{BaseMetadata, Icons, Implementation};

        let server_info = Implementation {
            base: BaseMetadata {
                name: "test".to_string(),
                title: None,
            },
            icons: Icons::default(),
            version: "0.1.0".to_string(),
            website_url: None,
            description: None,
        };
        let server = Arc::new(McpServer::new(server_info, ServerOptions::default()));
        let state = AxumHandlerState::new(server, AxumHandlerConfig::default());

        assert_eq!(state.session_manager().session_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcaster_creation() {
        use crate::server::{McpServer, ServerOptions};
        use mcp_core::types::{BaseMetadata, Icons, Implementation};

        let server_info = Implementation {
            base: BaseMetadata {
                name: "test".to_string(),
                title: None,
            },
            icons: Icons::default(),
            version: "0.1.0".to_string(),
            website_url: None,
            description: None,
        };
        let server = Arc::new(McpServer::new(server_info, ServerOptions::default()));
        let state = AxumHandlerState::new(server, AxumHandlerConfig::default());

        let broadcaster = state.get_or_create_broadcaster("session-1").await;
        assert_eq!(broadcaster.receiver_count(), 0);

        // Getting the same session should return the same broadcaster
        let broadcaster2 = state.get_or_create_broadcaster("session-1").await;
        assert_eq!(Arc::as_ptr(&broadcaster), Arc::as_ptr(&broadcaster2));
    }
}
