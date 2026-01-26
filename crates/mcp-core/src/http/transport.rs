//! Async transport trait for HTTP-based transports.

use async_trait::async_trait;

use crate::stdio::JsonRpcMessage;

/// Async transport trait for HTTP-based transports.
///
/// This trait defines the interface for HTTP transports that support
/// bidirectional communication via HTTP POST (for sending) and SSE (for receiving).
#[async_trait]
pub trait AsyncTransport: Send + Sync {
    /// The error type returned by transport operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Start the transport and establish connection.
    ///
    /// For clients, this typically involves:
    /// - Establishing an SSE connection for receiving messages
    /// - Optionally negotiating a session ID
    ///
    /// For servers, this is typically a no-op as they wait for connections.
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Send a JSON-RPC message.
    ///
    /// For clients, this sends an HTTP POST request.
    /// For servers, this sends an SSE event or queues it for delivery.
    async fn send(&self, message: &JsonRpcMessage) -> Result<(), Self::Error>;

    /// Close the transport and release resources.
    async fn close(&mut self) -> Result<(), Self::Error>;

    /// Get the current session ID, if established.
    fn session_id(&self) -> Option<&str>;
}

/// Trait for transports that can receive messages via callbacks.
///
/// This is similar to the event handler pattern used in `StdioClientTransport`.
pub trait MessageReceiver {
    /// The error type for transport errors.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Register a handler for incoming JSON-RPC messages.
    fn on_message<F>(&mut self, handler: F)
    where
        F: Fn(JsonRpcMessage) + Send + Sync + 'static;

    /// Register a handler for transport errors.
    fn on_error<F>(&mut self, handler: F)
    where
        F: Fn(Self::Error) + Send + Sync + 'static;

    /// Register a handler for connection close events.
    fn on_close<F>(&mut self, handler: F)
    where
        F: Fn() + Send + Sync + 'static;
}

/// Connection state for HTTP transports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Transport is not connected.
    Disconnected,
    /// Transport is connecting.
    Connecting,
    /// Transport is connected and ready.
    Connected,
    /// Transport is attempting to reconnect.
    Reconnecting,
    /// Transport has been closed.
    Closed,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Reconnecting => write!(f, "reconnecting"),
            Self::Closed => write!(f, "closed"),
        }
    }
}
