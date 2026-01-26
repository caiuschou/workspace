//! OAuth authentication module for MCP client.
//!
//! This module provides OAuth 2.1 client authentication support, including:
//!
//! - Authorization server metadata discovery (RFC 8414)
//! - Protected resource metadata discovery (RFC 9728)
//! - Dynamic client registration (RFC 7591)
//! - PKCE support (RFC 7636)
//! - Token refresh
//!
//! ## Example
//!
//! ```ignore
//! use mcp_client::auth::{
//!     OAuthClientProvider, InMemoryOAuthClientProvider, auth, AuthOptions,
//! };
//! use mcp_core::auth::OAuthClientMetadata;
//!
//! // Create a client provider
//! let metadata = OAuthClientMetadata {
//!     redirect_uris: vec!["http://localhost:8080/callback".to_string()],
//!     client_name: Some("My App".to_string()),
//!     ..Default::default()
//! };
//!
//! let provider = InMemoryOAuthClientProvider::new(
//!     Some("http://localhost:8080/callback".to_string()),
//!     metadata,
//! );
//!
//! // Run the auth flow
//! let result = auth(&provider, AuthOptions::new("https://api.example.com/mcp")).await?;
//! ```

mod discovery;
mod flow;
mod provider;

pub use discovery::{
    discover_authorization_server_metadata, discover_protected_resource_metadata,
    get_protected_resource_metadata_url,
};
pub use flow::{auth, register_client, start_authorization, AuthOptions};
pub use provider::{
    AuthResult, InMemoryOAuthClientProvider, InvalidationScope, OAuthClientError,
    OAuthClientProvider,
};
