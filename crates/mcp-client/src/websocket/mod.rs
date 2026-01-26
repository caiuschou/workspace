//! WebSocket client transport for MCP.
//!
//! This module provides a WebSocket-based transport for MCP client communication,
//! offering full-duplex bidirectional communication.

#[cfg(feature = "websocket")]
mod transport;
#[cfg(feature = "websocket")]
mod error;

#[cfg(feature = "websocket")]
pub use transport::WebSocketClientTransport;
#[cfg(feature = "websocket")]
pub use error::WebSocketClientError;
