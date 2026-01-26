//! HTTP transport module for MCP server.
//!
//! This module provides HTTP-based transport for the MCP server, supporting:
//! - POST requests for sending JSON-RPC messages
//! - GET requests for establishing SSE connections
//! - DELETE requests for closing sessions
//!
//! ## Features
//!
//! - `axum`: Enable axum integration with true SSE streaming and bidirectional communication.
//!   Use [`axum_handler::create_router`] to create an axum router.
//!
//! ## Example (with axum feature)
//!
//! ```ignore
//! use std::sync::Arc;
//! use mcp_server::http::axum_handler::{AxumHandlerConfig, AxumHandlerState, create_router};
//!
//! let state = Arc::new(AxumHandlerState::new(server, AxumHandlerConfig::default()));
//! let app = create_router(state);
//! // Start with axum::serve(...)
//! ```

mod broadcast;
#[cfg(feature = "axum")]
mod dns_protection;
mod error;
mod handler;
mod legacy_sse;
mod session_manager;
mod sse_writer;

#[cfg(feature = "axum")]
pub mod axum_handler;

pub use broadcast::{BufferedEvent, EventBuffer, EventBufferConfig};
pub use error::HttpServerError;
pub use handler::{HttpResponse, HttpServerHandler, HttpServerOptions, RequestHeaders};
pub use legacy_sse::{LegacySseConfig, LegacySseState, generate_session_id};
pub use session_manager::{SessionConfig, SessionManager, SessionState};
pub use sse_writer::{SseResponseBuilder, SseWriter};

#[cfg(feature = "tokio")]
pub use broadcast::async_broadcast::SseBroadcaster;

#[cfg(feature = "axum")]
pub use dns_protection::{
    host_header_validation, localhost_host_validation, DnsProtectionConfig, DnsProtectionLayer,
    DnsProtectionService,
};
#[cfg(feature = "axum")]
pub use legacy_sse::create_legacy_sse_router;
