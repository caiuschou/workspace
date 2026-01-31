//! Error types for the OpenCode SDK.

use thiserror::Error;

/// Errors that can occur when using the OpenCode SDK.
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// OpenCode command not found in PATH.
    #[error("opencode command not found: {0}. Enable auto_install or see https://opencode.ai/install")]
    CommandNotFound(String),

    /// Auto-install failed.
    #[error("failed to install opencode: {0}")]
    InstallFailed(String),

    /// Timeout waiting for AI response.
    #[error("timeout waiting for AI response after {timeout_ms}ms")]
    WaitResponseTimeout { timeout_ms: u64 },

    /// Failed to spawn server process.
    #[error("failed to spawn opencode serve: {0}")]
    SpawnFailed(#[source] std::io::Error),

    /// Server did not become ready within timeout.
    #[error("opencode server at {url} did not become ready within {timeout_ms}ms")]
    StartupTimeout { url: String, timeout_ms: u64 },
}
