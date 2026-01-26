//! OAuth 2.1 type definitions.
//!
//! This module defines types for OAuth 2.1 authentication and authorization,
//! including metadata structures defined in various RFCs:
//!
//! - RFC 8414: OAuth 2.0 Authorization Server Metadata
//! - RFC 9728: OAuth 2.0 Protected Resource Metadata
//! - RFC 7591: OAuth 2.0 Dynamic Client Registration
//! - RFC 7009: OAuth 2.0 Token Revocation

use serde::{Deserialize, Serialize};

/// RFC 8414: OAuth 2.0 Authorization Server Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthMetadata {
    /// The authorization server's issuer identifier.
    pub issuer: String,

    /// URL of the authorization server's authorization endpoint.
    pub authorization_endpoint: String,

    /// URL of the authorization server's token endpoint.
    pub token_endpoint: String,

    /// URL of the authorization server's dynamic client registration endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,

    /// JSON array containing a list of the OAuth 2.0 scope values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,

    /// JSON array containing a list of the OAuth 2.0 response_type values.
    pub response_types_supported: Vec<String>,

    /// JSON array containing a list of the OAuth 2.0 response_mode values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modes_supported: Option<Vec<String>>,

    /// JSON array containing a list of the OAuth 2.0 grant type values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types_supported: Option<Vec<String>>,

    /// JSON array containing a list of client authentication methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,

    /// JSON array containing a list of the JWS signing algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_signing_alg_values_supported: Option<Vec<String>>,

    /// URL of a page with human-readable documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_documentation: Option<String>,

    /// URL of the authorization server's OAuth 2.0 revocation endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,

    /// JSON array containing a list of client authentication methods for revocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint_auth_methods_supported: Option<Vec<String>>,

    /// URL of the authorization server's OAuth 2.0 introspection endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,

    /// PKCE code challenge methods supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_methods_supported: Option<Vec<String>>,

    /// Whether URL-based client IDs are supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id_metadata_document_supported: Option<bool>,
}

/// RFC 9728: OAuth 2.0 Protected Resource Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProtectedResourceMetadata {
    /// The protected resource's resource identifier.
    pub resource: String,

    /// JSON array containing the issuer identifiers of authorization servers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_servers: Option<Vec<String>>,

    /// URL of the protected resource's JWK Set document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,

    /// JSON array containing a list of the OAuth 2.0 scope values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,

    /// JSON array containing a list of bearer methods supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_methods_supported: Option<Vec<String>>,

    /// Human-readable name for the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,

    /// URL of a page with human-readable documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_documentation: Option<String>,
}

/// OAuth 2.1 token response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    /// The access token issued by the authorization server.
    pub access_token: String,

    /// The type of the token issued (e.g., "Bearer").
    pub token_type: String,

    /// The lifetime in seconds of the access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,

    /// The refresh token, which can be used to obtain new access tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// The scope of the access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// ID token (for OpenID Connect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}

/// RFC 7591: OAuth 2.0 Dynamic Client Registration - Client Metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OAuthClientMetadata {
    /// Array of redirection URI strings.
    pub redirect_uris: Vec<String>,

    /// Requested authentication method for the token endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,

    /// Array of OAuth 2.0 grant types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types: Option<Vec<String>>,

    /// Array of the OAuth 2.0 response types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_types: Option<Vec<String>>,

    /// Human-readable name of the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,

    /// URL of a web page providing information about the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,

    /// URL that references a logo for the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,

    /// Space-separated list of scope values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Array of e-mail addresses of people responsible for this client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<String>>,

    /// URL that points to a human-readable terms of service document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tos_uri: Option<String>,

    /// URL that points to a human-readable privacy policy document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_uri: Option<String>,

    /// URL referencing the client's JSON Web Key Set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,

    /// Client's JSON Web Key Set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks: Option<serde_json::Value>,

    /// A unique identifier string assigned by the client developer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_id: Option<String>,

    /// A version identifier string for the client software.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,
}

/// RFC 7591: OAuth 2.0 Dynamic Client Registration - Client Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClientInformation {
    /// OAuth 2.0 client identifier.
    pub client_id: String,

    /// OAuth 2.0 client secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// Time at which the client identifier was issued.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id_issued_at: Option<u64>,

    /// Time at which the client secret will expire.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret_expires_at: Option<u64>,
}

/// Full client information including both metadata and registration info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClientInformationFull {
    /// Client information fields.
    #[serde(flatten)]
    pub client_info: OAuthClientInformation,

    /// Client metadata fields.
    #[serde(flatten)]
    pub metadata: OAuthClientMetadata,

    /// Token endpoint auth method assigned by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,
}

/// RFC 7009: OAuth 2.0 Token Revocation - Revocation Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenRevocationRequest {
    /// The token that the client wants to get revoked.
    pub token: String,

    /// A hint about the type of the token submitted for revocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type_hint: Option<String>,
}

/// OAuth error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthErrorResponse {
    /// A single ASCII error code.
    pub error: String,

    /// Human-readable ASCII text description of the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,

    /// A URI identifying a human-readable web page with error info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

/// Information about a validated access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    /// The access token.
    pub token: String,

    /// The client ID the token was issued to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// The scopes associated with the token.
    pub scopes: Vec<String>,

    /// Unix timestamp when the token expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,

    /// Additional claims or metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl AuthInfo {
    /// Create a new AuthInfo with the given token.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            client_id: None,
            scopes: Vec::new(),
            expires_at: None,
            extra: None,
        }
    }

    /// Set the client ID.
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Set the scopes.
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Set the expiration time.
    pub fn with_expires_at(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            expires_at < now
        } else {
            false
        }
    }

    /// Check if the token has the required scopes.
    pub fn has_scopes(&self, required: &[&str]) -> bool {
        required.iter().all(|s| self.scopes.iter().any(|scope| scope == *s))
    }
}

/// Authorization parameters for starting the OAuth flow.
#[derive(Debug, Clone)]
pub struct AuthorizationParams {
    /// Optional state parameter for CSRF protection.
    pub state: Option<String>,

    /// Requested scopes.
    pub scopes: Option<Vec<String>>,

    /// PKCE code challenge.
    pub code_challenge: String,

    /// Redirect URI.
    pub redirect_uri: String,

    /// Resource indicator (RFC 8707).
    pub resource: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_metadata_serialization() {
        let metadata = OAuthMetadata {
            issuer: "https://auth.example.com".to_string(),
            authorization_endpoint: "https://auth.example.com/authorize".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            registration_endpoint: Some("https://auth.example.com/register".to_string()),
            scopes_supported: Some(vec!["openid".to_string(), "profile".to_string()]),
            response_types_supported: vec!["code".to_string()],
            response_modes_supported: None,
            grant_types_supported: Some(vec!["authorization_code".to_string(), "refresh_token".to_string()]),
            token_endpoint_auth_methods_supported: Some(vec!["client_secret_post".to_string()]),
            token_endpoint_auth_signing_alg_values_supported: None,
            service_documentation: None,
            revocation_endpoint: None,
            revocation_endpoint_auth_methods_supported: None,
            introspection_endpoint: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
            client_id_metadata_document_supported: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: OAuthMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.issuer, metadata.issuer);
        assert_eq!(deserialized.authorization_endpoint, metadata.authorization_endpoint);
    }

    #[test]
    fn test_auth_info_scopes() {
        let auth = AuthInfo::new("token123")
            .with_scopes(vec!["read".to_string(), "write".to_string()]);

        assert!(auth.has_scopes(&["read"]));
        assert!(auth.has_scopes(&["read", "write"]));
        assert!(!auth.has_scopes(&["admin"]));
    }

    #[test]
    fn test_auth_info_expiration() {
        let auth = AuthInfo::new("token123")
            .with_expires_at(0); // Already expired

        assert!(auth.is_expired());

        let auth = AuthInfo::new("token123")
            .with_expires_at(u64::MAX); // Never expires

        assert!(!auth.is_expired());
    }
}
