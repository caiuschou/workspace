//! WebSocket client error types.

use std::io;

/// Errors that can occur in the WebSocket client transport.
#[derive(Debug, thiserror::Error)]
pub enum WebSocketClientError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Transport already started")]
    AlreadyStarted,

    #[error("Not connected")]
    NotConnected,

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Send error: {0}")]
    Send(String),
}
