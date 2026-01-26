//! Error types for HTTP transport.

use thiserror::Error;

/// Errors that can occur during HTTP transport operations.
#[derive(Debug, Error)]
pub enum HttpTransportError {
    /// Connection to the server failed.
    #[error("connection failed: {0}")]
    Connection(String),

    /// The session has expired.
    #[error("session expired: {session_id}")]
    SessionExpired { session_id: String },

    /// The session ID is invalid or unknown.
    #[error("invalid session: {0}")]
    InvalidSession(String),

    /// JSON serialization or deserialization failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Failed to parse an SSE event.
    #[error("SSE parse error: {0}")]
    SseParse(String),

    /// The operation timed out.
    #[error("timeout")]
    Timeout,

    /// The transport has been closed.
    #[error("transport closed")]
    Closed,

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),
}
