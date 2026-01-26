use thiserror::Error;

use mcp_core::protocol::ProtocolError;

/// Errors emitted by the MCP server wrapper.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("capabilities are locked after initialization")]
    CapabilitiesLocked,

    #[error("capability not available: {0}")]
    Capability(String),

    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("handler error: {0}")]
    Handler(String),
}
