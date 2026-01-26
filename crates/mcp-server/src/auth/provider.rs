//! OAuth server provider interface.
//!
//! This module defines the trait for implementing OAuth server functionality.

use async_trait::async_trait;

use mcp_core::auth::{
    AuthInfo, AuthorizationParams, OAuthClientInformationFull, OAuthTokenRevocationRequest,
    OAuthTokens,
};

use super::clients::OAuthRegisteredClientsStore;

/// Error type for OAuth provider operations.
#[derive(Debug, thiserror::Error)]
pub enum OAuthProviderError {
    /// Invalid request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Invalid client.
    #[error("invalid client: {0}")]
    InvalidClient(String),

    /// Invalid grant.
    #[error("invalid grant: {0}")]
    InvalidGrant(String),

    /// Unauthorized client.
    #[error("unauthorized client: {0}")]
    UnauthorizedClient(String),

    /// Invalid scope.
    #[error("invalid scope: {0}")]
    InvalidScope(String),

    /// Access denied.
    #[error("access denied: {0}")]
    AccessDenied(String),

    /// Invalid token.
    #[error("invalid token: {0}")]
    InvalidToken(String),

    /// Server error.
    #[error("server error: {0}")]
    Server(String),
}

impl OAuthProviderError {
    /// Get the OAuth error code for this error.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::InvalidClient(_) => "invalid_client",
            Self::InvalidGrant(_) => "invalid_grant",
            Self::UnauthorizedClient(_) => "unauthorized_client",
            Self::InvalidScope(_) => "invalid_scope",
            Self::AccessDenied(_) => "access_denied",
            Self::InvalidToken(_) => "invalid_token",
            Self::Server(_) => "server_error",
        }
    }
}

/// Response type for authorization.
pub enum AuthorizeResponse {
    /// Redirect to a URL with authorization code.
    Redirect {
        /// The redirect URL with query parameters.
        url: String,
    },
    /// Return an HTML page (for login forms, consent screens, etc.).
    Html {
        /// The HTML content.
        content: String,
    },
    /// Return an error.
    Error {
        /// Error code.
        error: String,
        /// Error description.
        description: Option<String>,
    },
}

/// Trait for implementing an OAuth 2.1 authorization server.
#[async_trait]
pub trait OAuthServerProvider: Send + Sync {
    /// Get the client store.
    fn clients_store(&self) -> &dyn OAuthRegisteredClientsStore;

    /// Begin the authorization flow.
    ///
    /// This method should either:
    /// - Redirect to a login/consent page
    /// - Return an HTML page for the authorization UI
    /// - Redirect back to the client with an authorization code
    async fn authorize(
        &self,
        client: &OAuthClientInformationFull,
        params: AuthorizationParams,
    ) -> Result<AuthorizeResponse, OAuthProviderError>;

    /// Get the PKCE code challenge for an authorization code.
    async fn challenge_for_authorization_code(
        &self,
        client: &OAuthClientInformationFull,
        authorization_code: &str,
    ) -> Result<String, OAuthProviderError>;

    /// Exchange an authorization code for tokens.
    async fn exchange_authorization_code(
        &self,
        client: &OAuthClientInformationFull,
        authorization_code: &str,
        code_verifier: Option<&str>,
        redirect_uri: Option<&str>,
        resource: Option<&str>,
    ) -> Result<OAuthTokens, OAuthProviderError>;

    /// Exchange a refresh token for new tokens.
    async fn exchange_refresh_token(
        &self,
        client: &OAuthClientInformationFull,
        refresh_token: &str,
        scopes: Option<&[String]>,
        resource: Option<&str>,
    ) -> Result<OAuthTokens, OAuthProviderError>;

    /// Verify an access token and return information about it.
    async fn verify_access_token(&self, token: &str) -> Result<AuthInfo, OAuthProviderError>;

    /// Revoke a token.
    ///
    /// Default implementation does nothing (revocation not supported).
    async fn revoke_token(
        &self,
        _client: &OAuthClientInformationFull,
        _request: OAuthTokenRevocationRequest,
    ) -> Result<(), OAuthProviderError> {
        Ok(())
    }

    /// Whether to skip local PKCE validation.
    ///
    /// If true, the server will not perform PKCE validation locally and will
    /// pass the code_verifier to the upstream server.
    fn skip_local_pkce_validation(&self) -> bool {
        false
    }
}

/// Simplified trait for token verification only.
#[async_trait]
pub trait OAuthTokenVerifier: Send + Sync {
    /// Verify an access token and return information about it.
    async fn verify_access_token(&self, token: &str) -> Result<AuthInfo, OAuthProviderError>;
}

// Blanket implementation for full providers
#[async_trait]
impl<T: OAuthServerProvider + ?Sized> OAuthTokenVerifier for T {
    async fn verify_access_token(&self, token: &str) -> Result<AuthInfo, OAuthProviderError> {
        OAuthServerProvider::verify_access_token(self, token).await
    }
}
