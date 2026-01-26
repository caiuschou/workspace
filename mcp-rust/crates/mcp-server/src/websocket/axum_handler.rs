//! Axum WebSocket handler for MCP server.
//!
//! Provides a full-duplex WebSocket transport for MCP communication.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::{Any, CorsLayer};

use mcp_core::stdio::{deserialize_message, serialize_message, JsonRpcMessage};

use crate::server::McpServer;

/// MCP WebSocket subprotocol identifier.
pub const MCP_SUBPROTOCOL: &str = "mcp";

/// Configuration for the WebSocket handler.
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Endpoint path (default: "/ws").
    pub endpoint_path: String,
    /// Enable CORS.
    pub enable_cors: bool,
    /// Channel buffer size for outgoing messages.
    pub channel_buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            endpoint_path: "/ws".to_string(),
            enable_cors: true,
            channel_buffer_size: 100,
        }
    }
}

/// Per-connection state.
struct ConnectionState {
    /// Sender for outgoing messages.
    tx: mpsc::Sender<JsonRpcMessage>,
}

/// Shared state for the WebSocket handler.
pub struct WebSocketState {
    server: Arc<McpServer>,
    connections: RwLock<HashMap<String, ConnectionState>>,
    config: WebSocketConfig,
}

impl WebSocketState {
    /// Create a new WebSocket handler state.
    pub fn new(server: Arc<McpServer>, config: WebSocketConfig) -> Self {
        Self {
            server,
            connections: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Get the MCP server.
    pub fn server(&self) -> &Arc<McpServer> {
        &self.server
    }

    /// Get the configuration.
    pub fn config(&self) -> &WebSocketConfig {
        &self.config
    }

    /// Register a new connection.
    async fn register_connection(&self, connection_id: String, tx: mpsc::Sender<JsonRpcMessage>) {
        let mut connections = self.connections.write().await;
        connections.insert(connection_id, ConnectionState { tx });
    }

    /// Unregister a connection.
    async fn unregister_connection(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
    }

    /// Send a message to a specific connection.
    pub async fn send_to_connection(
        &self,
        connection_id: &str,
        message: JsonRpcMessage,
    ) -> Result<(), WebSocketError> {
        let connections = self.connections.read().await;
        if let Some(conn) = connections.get(connection_id) {
            conn.tx
                .send(message)
                .await
                .map_err(|_| WebSocketError::ConnectionClosed)?;
            Ok(())
        } else {
            Err(WebSocketError::ConnectionNotFound(connection_id.to_string()))
        }
    }

    /// Broadcast a message to all connections.
    pub async fn broadcast(&self, message: JsonRpcMessage) {
        let connections = self.connections.read().await;
        for conn in connections.values() {
            let _ = conn.tx.send(message.clone()).await;
        }
    }

    /// Get the number of active connections.
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

/// WebSocket error types.
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

/// Create an axum router for WebSocket MCP server.
pub fn create_websocket_router(state: Arc<WebSocketState>) -> Router {
    let mut router = Router::new()
        .route(&state.config.endpoint_path, get(handle_websocket_upgrade))
        .with_state(state.clone());

    if state.config.enable_cors {
        router = router.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET])
                .allow_headers([
                    header::UPGRADE,
                    header::CONNECTION,
                    header::SEC_WEBSOCKET_KEY,
                    header::SEC_WEBSOCKET_VERSION,
                    header::SEC_WEBSOCKET_PROTOCOL,
                ]),
        );
    }

    router
}

/// Handle WebSocket upgrade request.
async fn handle_websocket_upgrade(
    State(state): State<Arc<WebSocketState>>,
    ws: WebSocketUpgrade,
) -> Response {
    // Accept the WebSocket upgrade with MCP subprotocol
    ws.protocols([MCP_SUBPROTOCOL])
        .on_upgrade(move |socket| handle_websocket(state, socket))
}

/// Handle an established WebSocket connection.
pub async fn handle_websocket(state: Arc<WebSocketState>, socket: WebSocket) {
    // Generate a unique connection ID
    let connection_id = generate_connection_id();

    // Create channel for outgoing messages
    let (tx, rx) = mpsc::channel(state.config.channel_buffer_size);

    // Register the connection
    state.register_connection(connection_id.clone(), tx).await;

    // Split the WebSocket
    let (ws_sink, ws_stream) = socket.split();

    // Spawn tasks for reading and writing
    let read_task = tokio::spawn(handle_incoming(
        state.clone(),
        connection_id.clone(),
        ws_stream,
    ));

    let write_task = tokio::spawn(handle_outgoing(ws_sink, rx));

    // Wait for either task to complete
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    // Cleanup
    state.unregister_connection(&connection_id).await;
}

/// Handle incoming WebSocket messages.
async fn handle_incoming(
    state: Arc<WebSocketState>,
    connection_id: String,
    mut stream: SplitStream<WebSocket>,
) {
    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                if let Err(e) = process_message(&state, &connection_id, msg).await {
                    eprintln!("Error processing message: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("WebSocket receive error: {}", e);
                break;
            }
        }
    }
}

/// Process a single WebSocket message.
async fn process_message(
    state: &WebSocketState,
    connection_id: &str,
    msg: Message,
) -> Result<(), WebSocketError> {
    match msg {
        Message::Text(text) => {
            // Parse JSON-RPC message
            let message = deserialize_message(&text)
                .map_err(|e| WebSocketError::Serialization(e.to_string()))?;

            // Handle the message
            match message {
                JsonRpcMessage::Request(request) => {
                    let result = state
                        .server
                        .server()
                        .handle_request(request, Some(connection_id.to_string()))
                        .await;

                    match result {
                        Ok(response) => {
                            let response_msg = JsonRpcMessage::Result(response);
                            state.send_to_connection(connection_id, response_msg).await?;
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
                        .handle_notification(notification, Some(connection_id.to_string()))
                        .await;
                }
                JsonRpcMessage::Result(_) => {
                    // Unexpected result from client, ignore
                }
            }
        }
        Message::Binary(data) => {
            // Try to parse as JSON (some clients may send binary)
            if let Ok(text) = String::from_utf8(data) {
                return Box::pin(process_message(
                    state,
                    connection_id,
                    Message::Text(text.into()),
                ))
                .await;
            }
        }
        Message::Ping(_) | Message::Pong(_) => {
            // Handled automatically by axum
        }
        Message::Close(_) => {
            return Err(WebSocketError::ConnectionClosed);
        }
    }

    Ok(())
}

/// Handle outgoing WebSocket messages.
async fn handle_outgoing(
    mut sink: SplitSink<WebSocket, Message>,
    mut rx: mpsc::Receiver<JsonRpcMessage>,
) {
    while let Some(message) = rx.recv().await {
        match serialize_message(&message) {
            Ok(text) => {
                if sink.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Serialization error: {}", e);
            }
        }
    }
}

/// Generate a unique connection ID.
fn generate_connection_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("ws-{:x}", timestamp)
}

/// Create a JSON error response.
#[allow(dead_code)]
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
    use crate::server::{McpServer, ServerOptions};
    use mcp_core::types::{BaseMetadata, Icons, Implementation};

    #[tokio::test]
    async fn test_state_creation() {
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
        let state = WebSocketState::new(server, WebSocketConfig::default());

        assert_eq!(state.connection_count().await, 0);
    }

    #[test]
    fn test_connection_id_generation() {
        let id1 = generate_connection_id();
        let id2 = generate_connection_id();

        assert!(id1.starts_with("ws-"));
        assert!(id2.starts_with("ws-"));
        // IDs should be different (unless generated at exact same nanosecond)
        // This is a weak test but better than nothing
    }

    #[test]
    fn test_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.endpoint_path, "/ws");
        assert!(config.enable_cors);
        assert_eq!(config.channel_buffer_size, 100);
    }
}
