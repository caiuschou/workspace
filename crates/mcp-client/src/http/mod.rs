//! HTTP transport module for MCP client.
//!
//! This module provides an HTTP-based transport that uses POST for sending
//! messages and Server-Sent Events (SSE) for receiving.

mod config;
mod error;
mod legacy_sse;
mod reconnect;
mod sse_reader;
mod transport;

pub use config::HttpClientConfig;
pub use error::HttpClientError;
pub use legacy_sse::{LegacySseClientConfig, LegacySseClientTransport};
pub use reconnect::{ReconnectOptions, ReconnectState};
pub use sse_reader::SseReader;
pub use transport::HttpClientTransport;
