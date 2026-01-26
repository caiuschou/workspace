use std::{io, str};

use thiserror::Error;

/// Errors that can occur while managing the stdio transport.
#[derive(Debug, Error)]
pub enum StdioClientTransportError {
    #[error("transport already started")]
    AlreadyStarted,

    #[error("transport is not connected")]
    NotConnected,

    #[error("failed to spawn `{command}`: {source}")]
    Spawn {
        command: String,
        #[source]
        source: io::Error,
    },

    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error("UTF-8 error")]
    Utf8(#[from] str::Utf8Error),

    #[error("serialization failed")]
    Serialization(#[from] serde_json::Error),
}
