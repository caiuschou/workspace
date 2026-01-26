//! OAuth 2.1 error types.
//!
//! This module defines error types for OAuth 2.1 authentication and authorization,
//! following the error codes defined in:
//!
//! - RFC 6749: OAuth 2.0 Authorization Framework
//! - RFC 6750: Bearer Token Usage
//! - RFC 7009: Token Revocation
//! - RFC 7591: Dynamic Client Registration

use std::fmt;

use serde::{Deserialize, Serialize};

use super::types::OAuthErrorResponse;

/// Base trait for OAuth errors.
pub trait OAuthErrorCode {
    /// Returns the OAuth error code string.
    fn error_code(&self) -> &'static str;
}

/// OAuth error that can be converted to a response.
#[derive(Debug, Clone)]
pub struct OAuthError {
    /// The error code.
    pub code: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// URI for more information.
    pub uri: Option<String>,
}

impl OAuthError {
    /// Create a new OAuth error with the given code.
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: None,
            uri: None,
        }
    }

    /// Set the error description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the error URI.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Convert to an error response object.
    pub fn to_response(&self) -> OAuthErrorResponse {
        OAuthErrorResponse {
            error: self.code.clone(),
            error_description: self.description.clone(),
            error_uri: self.uri.clone(),
        }
    }
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for OAuthError {}

// Standard OAuth 2.0 error codes

/// Invalid request error - The request is missing a required parameter,
/// includes an invalid parameter value, or is otherwise malformed.
#[derive(Debug, Clone, Default)]
pub struct InvalidRequestError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl InvalidRequestError {
    pub const CODE: &'static str = "invalid_request";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for InvalidRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidRequestError {}

/// Invalid client error - Client authentication failed.
#[derive(Debug, Clone, Default)]
pub struct InvalidClientError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl InvalidClientError {
    pub const CODE: &'static str = "invalid_client";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for InvalidClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidClientError {}

/// Invalid grant error - The provided authorization grant or refresh token
/// is invalid, expired, revoked, or was issued to another client.
#[derive(Debug, Clone, Default)]
pub struct InvalidGrantError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl InvalidGrantError {
    pub const CODE: &'static str = "invalid_grant";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for InvalidGrantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidGrantError {}

/// Unauthorized client error - The authenticated client is not authorized
/// to use this authorization grant type.
#[derive(Debug, Clone, Default)]
pub struct UnauthorizedClientError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl UnauthorizedClientError {
    pub const CODE: &'static str = "unauthorized_client";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for UnauthorizedClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for UnauthorizedClientError {}

/// Unsupported grant type error - The authorization grant type is not
/// supported by the authorization server.
#[derive(Debug, Clone, Default)]
pub struct UnsupportedGrantTypeError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl UnsupportedGrantTypeError {
    pub const CODE: &'static str = "unsupported_grant_type";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for UnsupportedGrantTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for UnsupportedGrantTypeError {}

/// Invalid scope error - The requested scope is invalid, unknown, or malformed.
#[derive(Debug, Clone, Default)]
pub struct InvalidScopeError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl InvalidScopeError {
    pub const CODE: &'static str = "invalid_scope";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for InvalidScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidScopeError {}

/// Access denied error - The resource owner or authorization server denied the request.
#[derive(Debug, Clone, Default)]
pub struct AccessDeniedError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl AccessDeniedError {
    pub const CODE: &'static str = "access_denied";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for AccessDeniedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for AccessDeniedError {}

/// Server error - The authorization server encountered an unexpected condition.
#[derive(Debug, Clone, Default)]
pub struct ServerError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl ServerError {
    pub const CODE: &'static str = "server_error";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for ServerError {}

/// Temporarily unavailable error - The server is temporarily unable to handle the request.
#[derive(Debug, Clone, Default)]
pub struct TemporarilyUnavailableError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl TemporarilyUnavailableError {
    pub const CODE: &'static str = "temporarily_unavailable";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for TemporarilyUnavailableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for TemporarilyUnavailableError {}

/// Invalid token error (RFC 6750) - The access token provided is expired,
/// revoked, malformed, or invalid for other reasons.
#[derive(Debug, Clone, Default)]
pub struct InvalidTokenError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl InvalidTokenError {
    pub const CODE: &'static str = "invalid_token";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for InvalidTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidTokenError {}

/// Insufficient scope error (RFC 6750) - The request requires higher privileges
/// than provided by the access token.
#[derive(Debug, Clone, Default)]
pub struct InsufficientScopeError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl InsufficientScopeError {
    pub const CODE: &'static str = "insufficient_scope";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for InsufficientScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for InsufficientScopeError {}

/// Invalid client metadata error (RFC 7591) - The value of one or more
/// client metadata fields is invalid.
#[derive(Debug, Clone, Default)]
pub struct InvalidClientMetadataError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl InvalidClientMetadataError {
    pub const CODE: &'static str = "invalid_client_metadata";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for InvalidClientMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidClientMetadataError {}

/// Unsupported response type error - The authorization server does not
/// support obtaining an authorization code using this method.
#[derive(Debug, Clone, Default)]
pub struct UnsupportedResponseTypeError {
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl UnsupportedResponseTypeError {
    pub const CODE: &'static str = "unsupported_response_type";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn to_oauth_error(&self) -> OAuthError {
        let mut err = OAuthError::new(Self::CODE);
        if let Some(ref desc) = self.description {
            err = err.with_description(desc.clone());
        }
        if let Some(ref uri) = self.uri {
            err = err.with_uri(uri.clone());
        }
        err
    }
}

impl fmt::Display for UnsupportedResponseTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::CODE)?;
        if let Some(ref desc) = self.description {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl std::error::Error for UnsupportedResponseTypeError {}

/// Parse an OAuth error from an error response.
pub fn parse_oauth_error(response: &OAuthErrorResponse) -> OAuthError {
    OAuthError {
        code: response.error.clone(),
        description: response.error_description.clone(),
        uri: response.error_uri.clone(),
    }
}

/// OAuth error codes enum for pattern matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthErrorKind {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    AccessDenied,
    ServerError,
    TemporarilyUnavailable,
    InvalidToken,
    InsufficientScope,
    InvalidClientMetadata,
    UnsupportedResponseType,
    Unknown,
}

impl OAuthErrorKind {
    /// Parse error kind from error code string.
    pub fn from_code(code: &str) -> Self {
        match code {
            "invalid_request" => Self::InvalidRequest,
            "invalid_client" => Self::InvalidClient,
            "invalid_grant" => Self::InvalidGrant,
            "unauthorized_client" => Self::UnauthorizedClient,
            "unsupported_grant_type" => Self::UnsupportedGrantType,
            "invalid_scope" => Self::InvalidScope,
            "access_denied" => Self::AccessDenied,
            "server_error" => Self::ServerError,
            "temporarily_unavailable" => Self::TemporarilyUnavailable,
            "invalid_token" => Self::InvalidToken,
            "insufficient_scope" => Self::InsufficientScope,
            "invalid_client_metadata" => Self::InvalidClientMetadata,
            "unsupported_response_type" => Self::UnsupportedResponseType,
            _ => Self::Unknown,
        }
    }

    /// Get the error code string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidClient => "invalid_client",
            Self::InvalidGrant => "invalid_grant",
            Self::UnauthorizedClient => "unauthorized_client",
            Self::UnsupportedGrantType => "unsupported_grant_type",
            Self::InvalidScope => "invalid_scope",
            Self::AccessDenied => "access_denied",
            Self::ServerError => "server_error",
            Self::TemporarilyUnavailable => "temporarily_unavailable",
            Self::InvalidToken => "invalid_token",
            Self::InsufficientScope => "insufficient_scope",
            Self::InvalidClientMetadata => "invalid_client_metadata",
            Self::UnsupportedResponseType => "unsupported_response_type",
            Self::Unknown => "unknown_error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_error_display() {
        let err = OAuthError::new("invalid_request")
            .with_description("Missing required parameter");
        
        assert_eq!(format!("{}", err), "invalid_request: Missing required parameter");
    }

    #[test]
    fn test_oauth_error_response() {
        let err = InvalidClientError::new()
            .with_description("Unknown client");
        
        let response = err.to_oauth_error().to_response();
        
        assert_eq!(response.error, "invalid_client");
        assert_eq!(response.error_description, Some("Unknown client".to_string()));
    }

    #[test]
    fn test_error_kind_parsing() {
        assert_eq!(OAuthErrorKind::from_code("invalid_request"), OAuthErrorKind::InvalidRequest);
        assert_eq!(OAuthErrorKind::from_code("invalid_token"), OAuthErrorKind::InvalidToken);
        assert_eq!(OAuthErrorKind::from_code("unknown"), OAuthErrorKind::Unknown);
    }
}
