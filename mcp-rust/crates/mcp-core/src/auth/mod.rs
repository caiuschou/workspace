//! OAuth 2.1 authentication and authorization types.
//!
//! This module provides core types and error handling for OAuth 2.1 authentication,
//! including support for:
//!
//! - RFC 8414: OAuth 2.0 Authorization Server Metadata
//! - RFC 9728: OAuth 2.0 Protected Resource Metadata
//! - RFC 7591: OAuth 2.0 Dynamic Client Registration
//! - RFC 7009: OAuth 2.0 Token Revocation
//! - RFC 6750: OAuth 2.0 Bearer Token Usage

mod errors;
mod types;

pub use errors::{
    AccessDeniedError, InsufficientScopeError, InvalidClientError, InvalidClientMetadataError,
    InvalidGrantError, InvalidRequestError, InvalidScopeError, InvalidTokenError, OAuthError,
    OAuthErrorCode, OAuthErrorKind, ServerError, TemporarilyUnavailableError,
    UnauthorizedClientError, UnsupportedGrantTypeError, UnsupportedResponseTypeError,
    parse_oauth_error,
};
pub use types::{
    AuthInfo, AuthorizationParams, OAuthClientInformation, OAuthClientInformationFull,
    OAuthClientMetadata, OAuthErrorResponse, OAuthMetadata, OAuthProtectedResourceMetadata,
    OAuthTokenRevocationRequest, OAuthTokens,
};
