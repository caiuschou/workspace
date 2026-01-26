//! OAuth client provider interface.
//!
//! This module defines the trait for implementing OAuth client functionality.

use async_trait::async_trait;

use mcp_core::auth::{OAuthClientInformation, OAuthClientMetadata, OAuthTokens};

/// Error type for OAuth client provider operations.
#[derive(Debug, thiserror::Error)]
pub enum OAuthClientError {
    /// Invalid request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Invalid client.
    #[error("invalid client: {0}")]
    InvalidClient(String),

    /// Invalid grant.
    #[error("invalid grant: {0}")]
    InvalidGrant(String),

    /// Unauthorized.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Server error.
    #[error("server error: {0}")]
    Server(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),
}

/// Result type for authorization flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthResult {
    /// Authorization successful, tokens are available.
    Authorized,
    /// Redirect needed, call `redirect_to_authorization`.
    Redirect,
}

/// Trait for implementing an OAuth 2.1 client.
///
/// This trait provides a way to implement OAuth client functionality,
/// including token storage, client registration, and authorization flow handling.
#[async_trait]
pub trait OAuthClientProvider: Send + Sync {
    /// Get the redirect URL for the authorization flow.
    ///
    /// Return `None` for non-interactive flows (e.g., client_credentials, jwt-bearer).
    fn redirect_url(&self) -> Option<&str>;

    /// Get the external URL where the server can fetch client metadata.
    fn client_metadata_url(&self) -> Option<&str> {
        None
    }

    /// Get the client metadata.
    fn client_metadata(&self) -> &OAuthClientMetadata;

    /// Generate an OAuth state parameter for CSRF protection.
    async fn state(&self) -> Option<String> {
        None
    }

    /// Load saved client information.
    async fn client_information(&self) -> Option<OAuthClientInformation>;

    /// Save client information after dynamic registration.
    async fn save_client_information(&self, info: OAuthClientInformation) -> Result<(), OAuthClientError>;

    /// Load saved OAuth tokens.
    async fn tokens(&self) -> Option<OAuthTokens>;

    /// Save OAuth tokens.
    async fn save_tokens(&self, tokens: OAuthTokens) -> Result<(), OAuthClientError>;

    /// Redirect the user to the authorization URL.
    async fn redirect_to_authorization(&self, url: &str) -> Result<(), OAuthClientError>;

    /// Save the PKCE code verifier.
    async fn save_code_verifier(&self, verifier: String) -> Result<(), OAuthClientError>;

    /// Load the PKCE code verifier.
    async fn code_verifier(&self) -> Result<String, OAuthClientError>;

    /// Invalidate credentials.
    ///
    /// Called when credentials are determined to be invalid (e.g., after receiving
    /// an invalid_client or invalid_grant error).
    async fn invalidate_credentials(&self, scope: InvalidationScope) -> Result<(), OAuthClientError> {
        let _ = scope;
        Ok(())
    }

    /// Add custom client authentication to token requests.
    ///
    /// This optional method allows implementations to customize how client credentials
    /// are included in token requests. By default, the standard authentication methods
    /// (client_secret_post, client_secret_basic, none) are used.
    async fn add_client_authentication(
        &self,
        _headers: &mut Vec<(String, String)>,
        _params: &mut Vec<(String, String)>,
        _url: &str,
    ) -> Result<(), OAuthClientError> {
        Ok(())
    }

    /// Validate the resource URL for RFC 8707 resource indicators.
    ///
    /// By default, this validates that the resource matches the server URL.
    async fn validate_resource_url(
        &self,
        _server_url: &str,
        resource: Option<&str>,
    ) -> Result<Option<String>, OAuthClientError> {
        // Default: use the resource from metadata if provided
        Ok(resource.map(|s| s.to_string()))
    }

    /// Prepare grant-specific parameters for a token request.
    ///
    /// Override this for custom grant types (e.g., client_credentials, jwt-bearer).
    async fn prepare_token_request(&self, _scope: Option<&str>) -> Option<Vec<(String, String)>> {
        None
    }
}

/// Scope of credential invalidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationScope {
    /// Invalidate all credentials (client info, tokens, verifier).
    All,
    /// Invalidate only client information.
    Client,
    /// Invalidate only tokens.
    Tokens,
    /// Invalidate only the code verifier.
    Verifier,
}

/// A simple in-memory OAuth client provider for testing.
pub struct InMemoryOAuthClientProvider {
    redirect_url: Option<String>,
    client_metadata: OAuthClientMetadata,
    client_info: std::sync::RwLock<Option<OAuthClientInformation>>,
    tokens: std::sync::RwLock<Option<OAuthTokens>>,
    code_verifier: std::sync::RwLock<Option<String>>,
    authorization_url: std::sync::RwLock<Option<String>>,
}

impl InMemoryOAuthClientProvider {
    /// Create a new in-memory provider.
    pub fn new(redirect_url: Option<String>, client_metadata: OAuthClientMetadata) -> Self {
        Self {
            redirect_url,
            client_metadata,
            client_info: std::sync::RwLock::new(None),
            tokens: std::sync::RwLock::new(None),
            code_verifier: std::sync::RwLock::new(None),
            authorization_url: std::sync::RwLock::new(None),
        }
    }

    /// Set pre-registered client information.
    pub fn with_client_info(self, info: OAuthClientInformation) -> Self {
        *self.client_info.write().unwrap() = Some(info);
        self
    }

    /// Get the last authorization URL that was set.
    pub fn get_authorization_url(&self) -> Option<String> {
        self.authorization_url.read().unwrap().clone()
    }
}

#[async_trait]
impl OAuthClientProvider for InMemoryOAuthClientProvider {
    fn redirect_url(&self) -> Option<&str> {
        self.redirect_url.as_deref()
    }

    fn client_metadata(&self) -> &OAuthClientMetadata {
        &self.client_metadata
    }

    async fn client_information(&self) -> Option<OAuthClientInformation> {
        self.client_info.read().unwrap().clone()
    }

    async fn save_client_information(&self, info: OAuthClientInformation) -> Result<(), OAuthClientError> {
        *self.client_info.write().unwrap() = Some(info);
        Ok(())
    }

    async fn tokens(&self) -> Option<OAuthTokens> {
        self.tokens.read().unwrap().clone()
    }

    async fn save_tokens(&self, tokens: OAuthTokens) -> Result<(), OAuthClientError> {
        *self.tokens.write().unwrap() = Some(tokens);
        Ok(())
    }

    async fn redirect_to_authorization(&self, url: &str) -> Result<(), OAuthClientError> {
        *self.authorization_url.write().unwrap() = Some(url.to_string());
        Ok(())
    }

    async fn save_code_verifier(&self, verifier: String) -> Result<(), OAuthClientError> {
        *self.code_verifier.write().unwrap() = Some(verifier);
        Ok(())
    }

    async fn code_verifier(&self) -> Result<String, OAuthClientError> {
        self.code_verifier
            .read()
            .unwrap()
            .clone()
            .ok_or_else(|| OAuthClientError::Storage("No code verifier saved".to_string()))
    }

    async fn invalidate_credentials(&self, scope: InvalidationScope) -> Result<(), OAuthClientError> {
        match scope {
            InvalidationScope::All => {
                *self.client_info.write().unwrap() = None;
                *self.tokens.write().unwrap() = None;
                *self.code_verifier.write().unwrap() = None;
            }
            InvalidationScope::Client => {
                *self.client_info.write().unwrap() = None;
            }
            InvalidationScope::Tokens => {
                *self.tokens.write().unwrap() = None;
            }
            InvalidationScope::Verifier => {
                *self.code_verifier.write().unwrap() = None;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_provider() {
        let metadata = OAuthClientMetadata {
            redirect_uris: vec!["http://localhost:8080/callback".to_string()],
            client_name: Some("Test Client".to_string()),
            ..Default::default()
        };

        let provider = InMemoryOAuthClientProvider::new(
            Some("http://localhost:8080/callback".to_string()),
            metadata,
        );

        // Save and retrieve client info
        let client_info = OAuthClientInformation {
            client_id: "test-client".to_string(),
            client_secret: Some("test-secret".to_string()),
            client_id_issued_at: None,
            client_secret_expires_at: None,
        };

        provider.save_client_information(client_info.clone()).await.unwrap();
        let retrieved = provider.client_information().await.unwrap();
        assert_eq!(retrieved.client_id, "test-client");

        // Save and retrieve tokens
        let tokens = OAuthTokens {
            access_token: "access123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("refresh456".to_string()),
            scope: None,
            id_token: None,
        };

        provider.save_tokens(tokens.clone()).await.unwrap();
        let retrieved = provider.tokens().await.unwrap();
        assert_eq!(retrieved.access_token, "access123");
    }
}
