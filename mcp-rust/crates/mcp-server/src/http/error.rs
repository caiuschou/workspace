//! Error types for HTTP server transport.

use mcp_core::http::HttpTransportError;
use thiserror::Error;

use crate::server::ServerError;

/// Errors that can occur during HTTP server operations.
#[derive(Debug, Error)]
pub enum HttpServerError {
    /// Core transport error.
    #[error("transport error: {0}")]
    Transport(#[from] HttpTransportError),

    /// Session limit has been reached.
    #[error("session limit reached (max: {max})")]
    SessionLimitReached { max: usize },

    /// Invalid HTTP request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Missing required header.
    #[error("missing header: {0}")]
    MissingHeader(String),

    /// Invalid header value.
    #[error("invalid header value for {header}: {message}")]
    InvalidHeader { header: String, message: String },

    /// Server error.
    #[error("server error: {0}")]
    Server(#[from] ServerError),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(String),

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Session expired.
    #[error("session expired: {0}")]
    SessionExpired(String),

    /// Method not allowed.
    #[error("method not allowed: {0}")]
    MethodNotAllowed(String),

    /// Unsupported content type.
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),
}

impl HttpServerError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::InvalidRequest(_) => 400,
            Self::MissingHeader(_) => 400,
            Self::InvalidHeader { .. } => 400,
            Self::UnsupportedContentType(_) => 415,
            Self::MethodNotAllowed(_) => 405,
            Self::SessionNotFound(_) => 404,
            Self::SessionExpired(_) => 410,
            Self::SessionLimitReached { .. } => 503,
            Self::Server(_) => 500,
            Self::Transport(_) => 500,
            Self::Serialization(_) => 500,
            Self::Io(_) => 500,
        }
    }
}
