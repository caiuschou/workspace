//! OAuth authentication module for MCP server.
//!
//! This module provides OAuth 2.1 authentication support for MCP servers,
//! including:
//!
//! - Authorization server endpoints (authorize, token, register, revoke)
//! - Metadata endpoints (RFC 8414, RFC 9728)
//! - Bearer token authentication middleware
//! - Client authentication middleware
//!
//! ## Features
//!
//! - `axum`: Enable axum integration for OAuth routes and middleware.
//!
//! ## Example
//!
//! ```ignore
//! use std::sync::Arc;
//! use mcp_server::auth::{
//!     create_oauth_router, OAuthRouterOptions, InMemoryClientStore,
//! };
//!
//! // Create an OAuth provider (you need to implement OAuthServerProvider)
//! let provider = Arc::new(MyOAuthProvider::new());
//!
//! // Create OAuth router
//! let oauth_router = create_oauth_router(
//!     provider,
//!     OAuthRouterOptions::new("https://auth.example.com"),
//! );
//!
//! // Add to your axum app
//! let app = Router::new()
//!     .merge(oauth_router)
//!     .merge(mcp_router);
//! ```

mod clients;
#[cfg(feature = "axum")]
mod handlers;
pub mod middleware;
mod provider;
#[cfg(feature = "axum")]
mod router;

pub use clients::{ClientStoreError, InMemoryClientStore, OAuthRegisteredClientsStore};
pub use provider::{AuthorizeResponse, OAuthProviderError, OAuthServerProvider, OAuthTokenVerifier};

#[cfg(feature = "axum")]
pub use router::{
    create_oauth_metadata, create_oauth_metadata_router, create_oauth_router,
    create_protected_resource_metadata, OAuthRouterOptions, OAuthRouterState,
};
