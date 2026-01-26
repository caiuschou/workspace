//! WebSocket client transport implementation.

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use mcp_core::http::ConnectionState;
use mcp_core::stdio::{deserialize_message, serialize_message, JsonRpcMessage};

use super::error::WebSocketClientError;

/// MCP WebSocket subprotocol identifier.
pub const MCP_SUBPROTOCOL: &str = "mcp";

type MessageHandler = Arc<dyn Fn(JsonRpcMessage) + Send + Sync>;
type ErrorHandler = Arc<dyn Fn(WebSocketClientError) + Send + Sync>;
type CloseHandler = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct EventHandlers {
    message: Option<MessageHandler>,
    error: Option<ErrorHandler>,
    close: Option<CloseHandler>,
}

/// WebSocket client transport for MCP communication.
///
/// This transport provides full-duplex communication over WebSocket.
///
/// # Example
///
/// ```ignore
/// use mcp_client::websocket::WebSocketClientTransport;
///
/// #[tokio::main]
/// async fn main() {
///     let mut transport = WebSocketClientTransport::new("ws://localhost:8080/ws");
///
///     transport
///         .on_message(|msg| println!("Received: {:?}", msg))
///         .on_error(|e| eprintln!("Error: {:?}", e))
///         .on_close(|| println!("Connection closed"));
///
///     transport.start().await.unwrap();
///     // Use transport.send() to send messages
///     transport.close().await.unwrap();
/// }
/// ```
pub struct WebSocketClientTransport {
    url: String,
    state: Arc<RwLock<ConnectionState>>,
    handlers: Arc<Mutex<EventHandlers>>,
    tx: Arc<RwLock<Option<mpsc::Sender<JsonRpcMessage>>>>,
    shutdown: Arc<RwLock<bool>>,
}

impl WebSocketClientTransport {
    /// Create a new WebSocket client transport.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            handlers: Arc::new(Mutex::new(EventHandlers::default())),
            tx: Arc::new(RwLock::new(None)),
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Register a handler for incoming JSON-RPC messages.
    pub fn on_message(
        &mut self,
        handler: impl Fn(JsonRpcMessage) + Send + Sync + 'static,
    ) -> &mut Self {
        let handlers = self.handlers.clone();
        tokio::spawn(async move {
            let mut guard = handlers.lock().await;
            guard.message = Some(Arc::new(handler));
        });
        self
    }

    /// Register a handler for transport errors.
    pub fn on_error(
        &mut self,
        handler: impl Fn(WebSocketClientError) + Send + Sync + 'static,
    ) -> &mut Self {
        let handlers = self.handlers.clone();
        tokio::spawn(async move {
            let mut guard = handlers.lock().await;
            guard.error = Some(Arc::new(handler));
        });
        self
    }

    /// Register a handler for connection close events.
    pub fn on_close(&mut self, handler: impl Fn() + Send + Sync + 'static) -> &mut Self {
        let handlers = self.handlers.clone();
        tokio::spawn(async move {
            let mut guard = handlers.lock().await;
            guard.close = Some(Arc::new(handler));
        });
        self
    }

    /// Get the current connection state.
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Start the transport and establish WebSocket connection.
    pub async fn start(&mut self) -> Result<(), WebSocketClientError> {
        if *self.state.read().await != ConnectionState::Disconnected {
            return Err(WebSocketClientError::AlreadyStarted);
        }

        *self.state.write().await = ConnectionState::Connecting;
        *self.shutdown.write().await = false;

        // Build request with subprotocol
        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&self.url)
            .header("Sec-WebSocket-Protocol", MCP_SUBPROTOCOL)
            .header("Host", extract_host(&self.url).unwrap_or_default())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .body(())
            .map_err(|e| WebSocketClientError::Connection(e.to_string()))?;

        // Connect
        let (ws_stream, _response) = connect_async(request)
            .await
            .map_err(|e| WebSocketClientError::Connection(e.to_string()))?;

        *self.state.write().await = ConnectionState::Connected;

        // Split the WebSocket
        let (ws_sink, ws_stream) = ws_stream.split();

        // Create channel for outgoing messages
        let (tx, rx) = mpsc::channel::<JsonRpcMessage>(100);
        *self.tx.write().await = Some(tx);

        // Spawn read task
        let handlers = Arc::clone(&self.handlers);
        let state = Arc::clone(&self.state);
        let shutdown = Arc::clone(&self.shutdown);

        tokio::spawn(async move {
            handle_incoming(ws_stream, handlers, state, shutdown).await;
        });

        // Spawn write task
        let shutdown_write = Arc::clone(&self.shutdown);
        tokio::spawn(async move {
            handle_outgoing(ws_sink, rx, shutdown_write).await;
        });

        Ok(())
    }

    /// Send a JSON-RPC message.
    pub async fn send(&self, message: &JsonRpcMessage) -> Result<(), WebSocketClientError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(WebSocketClientError::NotConnected);
        }

        let tx = self.tx.read().await;
        if let Some(ref sender) = *tx {
            sender
                .send(message.clone())
                .await
                .map_err(|e| WebSocketClientError::Send(e.to_string()))?;
            Ok(())
        } else {
            Err(WebSocketClientError::NotConnected)
        }
    }

    /// Close the transport.
    pub async fn close(&mut self) -> Result<(), WebSocketClientError> {
        *self.shutdown.write().await = true;
        *self.state.write().await = ConnectionState::Closed;

        // Drop the sender to signal write task to stop
        *self.tx.write().await = None;

        // Dispatch close event
        let handlers = self.handlers.lock().await;
        if let Some(ref handler) = handlers.close {
            handler();
        }

        Ok(())
    }
}

/// Handle incoming WebSocket messages.
async fn handle_incoming<S>(
    mut stream: S,
    handlers: Arc<Mutex<EventHandlers>>,
    state: Arc<RwLock<ConnectionState>>,
    shutdown: Arc<RwLock<bool>>,
) where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    while let Some(result) = stream.next().await {
        if *shutdown.read().await {
            break;
        }

        match result {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        match deserialize_message(&text) {
                            Ok(message) => {
                                let guard = handlers.lock().await;
                                if let Some(ref handler) = guard.message {
                                    handler(message);
                                }
                            }
                            Err(e) => {
                                let guard = handlers.lock().await;
                                if let Some(ref handler) = guard.error {
                                    handler(WebSocketClientError::Serialization(e));
                                }
                            }
                        }
                    }
                    Message::Binary(data) => {
                        // Try to parse as JSON
                        if let Ok(text) = String::from_utf8(data) {
                            if let Ok(message) = deserialize_message(&text) {
                                let guard = handlers.lock().await;
                                if let Some(ref handler) = guard.message {
                                    handler(message);
                                }
                            }
                        }
                    }
                    Message::Ping(_) | Message::Pong(_) => {
                        // Handled automatically by tungstenite
                    }
                    Message::Close(_) => {
                        break;
                    }
                    Message::Frame(_) => {
                        // Raw frames, ignore
                    }
                }
            }
            Err(e) => {
                let guard = handlers.lock().await;
                if let Some(ref handler) = guard.error {
                    handler(WebSocketClientError::WebSocket(e.to_string()));
                }
                break;
            }
        }
    }

    // Update state
    let mut s = state.write().await;
    if *s != ConnectionState::Closed {
        *s = ConnectionState::Disconnected;
    }

    // Dispatch close event
    let guard = handlers.lock().await;
    if let Some(ref handler) = guard.close {
        handler();
    }
}

/// Handle outgoing WebSocket messages.
async fn handle_outgoing<S>(
    mut sink: S,
    mut rx: mpsc::Receiver<JsonRpcMessage>,
    shutdown: Arc<RwLock<bool>>,
) where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    while let Some(message) = rx.recv().await {
        if *shutdown.read().await {
            break;
        }

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

    // Send close frame
    let _ = sink.send(Message::Close(None)).await;
}

/// Extract host from URL string.
fn extract_host(url: &str) -> Option<String> {
    // Simple extraction: ws://host:port/path or wss://host:port/path
    let without_scheme = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("wss://"))?;

    let host_port = without_scheme.split('/').next()?;
    Some(host_port.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_creation() {
        let transport = WebSocketClientTransport::new("ws://localhost:8080/ws");
        assert_eq!(transport.state().await, ConnectionState::Disconnected);
    }

    #[test]
    fn test_subprotocol() {
        assert_eq!(MCP_SUBPROTOCOL, "mcp");
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(
            extract_host("ws://localhost:8080/ws"),
            Some("localhost:8080".to_string())
        );
        assert_eq!(
            extract_host("wss://example.com/mcp"),
            Some("example.com".to_string())
        );
        assert_eq!(extract_host("http://invalid"), None);
    }
}
