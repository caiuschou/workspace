use thiserror::Error;

use crate::schema::ValidationError;

/// Errors that can occur inside the protocol runtime.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("no handler registered for method `{0}`")]
    UnknownMethod(String),

    #[error("request cancelled")]
    Cancelled,

    #[error("request timed out")]
    Timeout,

    #[error("capability check failed: {0}")]
    Capability(String),

    #[error("task support is not available")]
    TaskUnsupported,

    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error("handler failed: {0}")]
    Handler(String),

    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}
