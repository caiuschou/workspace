//! Error types for HTTP client transport.

use mcp_core::http::HttpTransportError;
use thiserror::Error;

/// Errors that can occur during HTTP client transport operations.
#[derive(Debug, Error)]
pub enum HttpClientError {
    /// The transport is already started.
    #[error("transport already started")]
    AlreadyStarted,

    /// The transport is not connected.
    #[error("transport is not connected")]
    NotConnected,

    /// Core transport error.
    #[error("transport error: {0}")]
    Transport(#[from] HttpTransportError),

    /// HTTP request failed with a status code.
    #[error("HTTP error: status {status}")]
    HttpStatus {
        /// The HTTP status code.
        status: u16,
        /// Optional response body.
        body: Option<String>,
    },

    /// HTTP request error.
    #[error("request error: {0}")]
    Request(String),

    /// Authentication failed.
    #[error("authentication failed: {0}")]
    Auth(String),

    /// All reconnection attempts exhausted.
    #[error("reconnection attempts exhausted")]
    ReconnectionExhausted,

    /// Invalid URL.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// SSE connection error.
    #[error("SSE error: {0}")]
    Sse(String),

    /// The transport has been closed.
    #[error("transport closed")]
    Closed,

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<HttpClientError> for HttpTransportError {
    fn from(err: HttpClientError) -> Self {
        match err {
            HttpClientError::Transport(e) => e,
            HttpClientError::HttpStatus { status, body } => {
                HttpTransportError::Connection(format!(
                    "HTTP {}: {}",
                    status,
                    body.unwrap_or_default()
                ))
            }
            HttpClientError::Request(msg) => HttpTransportError::Connection(msg),
            HttpClientError::Auth(msg) => HttpTransportError::Connection(format!("auth: {}", msg)),
            HttpClientError::ReconnectionExhausted => {
                HttpTransportError::Connection("reconnection exhausted".to_string())
            }
            HttpClientError::InvalidUrl(url) => {
                HttpTransportError::Connection(format!("invalid URL: {}", url))
            }
            HttpClientError::Serialization(e) => HttpTransportError::Serialization(e),
            HttpClientError::Sse(msg) => HttpTransportError::SseParse(msg),
            HttpClientError::Closed => HttpTransportError::Closed,
            HttpClientError::AlreadyStarted => {
                HttpTransportError::Connection("already started".to_string())
            }
            HttpClientError::NotConnected => {
                HttpTransportError::Connection("not connected".to_string())
            }
            HttpClientError::Io(e) => HttpTransportError::Io(e.to_string()),
        }
    }
}
