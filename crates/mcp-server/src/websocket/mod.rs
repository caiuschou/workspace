//! WebSocket transport for MCP server.
//!
//! This module provides WebSocket-based transport for MCP communication,
//! offering a full-duplex bidirectional channel.

#[cfg(feature = "websocket")]
mod axum_handler;

#[cfg(feature = "websocket")]
pub use axum_handler::{
    WebSocketConfig, WebSocketError, WebSocketState, create_websocket_router, handle_websocket,
};
