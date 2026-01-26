//! OAuth middleware for MCP server.
//!
//! This module provides middleware for OAuth authentication.

#[cfg(feature = "axum")]
mod bearer_auth;
#[cfg(feature = "axum")]
mod client_auth;

#[cfg(feature = "axum")]
pub use bearer_auth::{BearerAuthLayer, BearerAuthMiddleware, BearerAuthOptions};
#[cfg(feature = "axum")]
pub use client_auth::{ClientAuthLayer, ClientAuthMiddleware};
