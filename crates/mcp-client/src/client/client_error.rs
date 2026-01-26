use thiserror::Error;

use mcp_core::protocol::ProtocolError;

/// Errors that can occur while driving the client runtime.
#[derive(Debug, Error)]
pub enum ClientError<TransportError> {
    #[error("transport failed: {0}")]
    Transport(#[from] TransportError),

    #[error("protocol failed: {0}")]
    Protocol(ProtocolError),

    #[error("data serialization failed: {0}")]
    Serialization(serde_json::Error),

    #[error("initialization failed: {0}")]
    Initialization(String),

    #[error("capability mismatch: {0}")]
    Capability(String),

    #[error("validation failed: {0}")]
    Validation(String),
}
