//! OAuth router for axum.
//!
//! This module provides functions to create OAuth routes for an axum application.

#![cfg(feature = "axum")]

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use mcp_core::auth::{OAuthMetadata, OAuthProtectedResourceMetadata};

use super::handlers::{
    authorize_handler, metadata_handler, register_handler, revoke_handler, token_handler,
};
use super::provider::OAuthServerProvider;

/// Options for the OAuth router.
#[derive(Debug, Clone)]
pub struct OAuthRouterOptions {
    /// The authorization server's issuer identifier.
    /// Must be an HTTPS URL with no query or fragment components.
    pub issuer_url: String,

    /// The base URL for the OAuth endpoints.
    /// If not provided, the issuer URL is used.
    pub base_url: Option<String>,

    /// URL of documentation for developers.
    pub service_documentation_url: Option<String>,

    /// Scopes supported by this authorization server.
    pub scopes_supported: Option<Vec<String>>,

    /// The resource name to display in protected resource metadata.
    pub resource_name: Option<String>,

    /// The URL of the protected resource server.
    /// If not provided, falls back to base_url then issuer_url.
    pub resource_server_url: Option<String>,
}

impl OAuthRouterOptions {
    /// Create new options with the given issuer URL.
    pub fn new(issuer_url: impl Into<String>) -> Self {
        Self {
            issuer_url: issuer_url.into(),
            base_url: None,
            service_documentation_url: None,
            scopes_supported: None,
            resource_name: None,
            resource_server_url: None,
        }
    }

    /// Set the base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the service documentation URL.
    pub fn with_service_documentation(mut self, url: impl Into<String>) -> Self {
        self.service_documentation_url = Some(url.into());
        self
    }

    /// Set the supported scopes.
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes_supported = Some(scopes);
        self
    }

    /// Set the resource name.
    pub fn with_resource_name(mut self, name: impl Into<String>) -> Self {
        self.resource_name = Some(name.into());
        self
    }

    /// Set the resource server URL.
    pub fn with_resource_server_url(mut self, url: impl Into<String>) -> Self {
        self.resource_server_url = Some(url.into());
        self
    }
}

/// State for the OAuth router.
pub struct OAuthRouterState<P: OAuthServerProvider> {
    /// The OAuth provider.
    pub provider: Arc<P>,
    /// OAuth metadata.
    pub metadata: OAuthMetadata,
    /// Protected resource metadata.
    pub resource_metadata: OAuthProtectedResourceMetadata,
}

impl<P: OAuthServerProvider> Clone for OAuthRouterState<P> {
    fn clone(&self) -> Self {
        Self {
            provider: Arc::clone(&self.provider),
            metadata: self.metadata.clone(),
            resource_metadata: self.resource_metadata.clone(),
        }
    }
}

/// Create OAuth metadata from options.
pub fn create_oauth_metadata<P: OAuthServerProvider>(
    _provider: &P,
    options: &OAuthRouterOptions,
) -> OAuthMetadata {
    let base = options.base_url.as_ref().unwrap_or(&options.issuer_url);
    let base = base.trim_end_matches('/');

    // Check if the provider supports registration
    // We'll always include registration endpoint for simplicity
    let registration_endpoint = Some(format!("{}/register", base));

    OAuthMetadata {
        issuer: options.issuer_url.clone(),
        authorization_endpoint: format!("{}/authorize", base),
        token_endpoint: format!("{}/token", base),
        registration_endpoint,
        scopes_supported: options.scopes_supported.clone(),
        response_types_supported: vec!["code".to_string()],
        response_modes_supported: None,
        grant_types_supported: Some(vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ]),
        token_endpoint_auth_methods_supported: Some(vec![
            "client_secret_post".to_string(),
            "none".to_string(),
        ]),
        token_endpoint_auth_signing_alg_values_supported: None,
        service_documentation: options.service_documentation_url.clone(),
        revocation_endpoint: Some(format!("{}/revoke", base)),
        revocation_endpoint_auth_methods_supported: Some(vec!["client_secret_post".to_string()]),
        introspection_endpoint: None,
        code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        client_id_metadata_document_supported: None,
    }
}

/// Create protected resource metadata from options.
pub fn create_protected_resource_metadata(
    options: &OAuthRouterOptions,
) -> OAuthProtectedResourceMetadata {
    let resource_url = options
        .resource_server_url
        .as_ref()
        .or(options.base_url.as_ref())
        .unwrap_or(&options.issuer_url);

    OAuthProtectedResourceMetadata {
        resource: resource_url.clone(),
        authorization_servers: Some(vec![options.issuer_url.clone()]),
        jwks_uri: None,
        scopes_supported: options.scopes_supported.clone(),
        bearer_methods_supported: Some(vec!["header".to_string()]),
        resource_name: options.resource_name.clone(),
        resource_documentation: options.service_documentation_url.clone(),
    }
}

/// Create a full OAuth router with all endpoints.
///
/// This includes:
/// - `GET /authorize` - Authorization endpoint
/// - `POST /token` - Token endpoint
/// - `POST /register` - Dynamic client registration (RFC 7591)
/// - `POST /revoke` - Token revocation (RFC 7009)
/// - `GET /.well-known/oauth-authorization-server` - Server metadata (RFC 8414)
/// - `GET /.well-known/oauth-protected-resource` - Resource metadata (RFC 9728)
pub fn create_oauth_router<P: OAuthServerProvider + 'static>(
    provider: Arc<P>,
    options: OAuthRouterOptions,
) -> Router {
    let metadata = create_oauth_metadata(provider.as_ref(), &options);
    let resource_metadata = create_protected_resource_metadata(&options);

    let state = OAuthRouterState {
        provider,
        metadata: metadata.clone(),
        resource_metadata: resource_metadata.clone(),
    };

    Router::new()
        .route("/authorize", get(authorize_handler::<P>))
        .route("/token", post(token_handler::<P>))
        .route("/register", post(register_handler::<P>))
        .route("/revoke", post(revoke_handler::<P>))
        .route(
            "/.well-known/oauth-authorization-server",
            get(metadata_handler::<P>),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(resource_metadata_handler::<P>),
        )
        .with_state(Arc::new(state))
}

/// Create a metadata-only router.
///
/// Use this when your MCP server is only a resource server (not an authorization server)
/// and you want to advertise which authorization server to use.
pub fn create_oauth_metadata_router<P: OAuthServerProvider + 'static>(
    provider: Arc<P>,
    options: OAuthRouterOptions,
) -> Router {
    let metadata = create_oauth_metadata(provider.as_ref(), &options);
    let resource_metadata = create_protected_resource_metadata(&options);

    let state = OAuthRouterState {
        provider,
        metadata: metadata.clone(),
        resource_metadata: resource_metadata.clone(),
    };

    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(metadata_handler::<P>),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(resource_metadata_handler::<P>),
        )
        .with_state(Arc::new(state))
}

/// Handler for protected resource metadata.
async fn resource_metadata_handler<P: OAuthServerProvider + 'static>(
    axum::extract::State(state): axum::extract::State<Arc<OAuthRouterState<P>>>,
) -> axum::response::Json<OAuthProtectedResourceMetadata> {
    axum::response::Json(state.resource_metadata.clone())
}
