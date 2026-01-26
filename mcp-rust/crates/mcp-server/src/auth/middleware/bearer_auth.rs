//! Bearer token authentication middleware.
//!
//! This middleware validates Bearer tokens in the Authorization header
//! and adds authentication information to the request extensions.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{header, Request, Response, StatusCode};
use tower::{Layer, Service};

use mcp_core::auth::{AuthInfo, OAuthErrorResponse};

use crate::auth::provider::{OAuthProviderError, OAuthTokenVerifier};

/// Options for bearer authentication middleware.
#[derive(Debug, Clone)]
pub struct BearerAuthOptions {
    /// Required scopes for the token.
    pub required_scopes: Vec<String>,
    /// URL of the protected resource metadata for WWW-Authenticate header.
    pub resource_metadata_url: Option<String>,
}

impl Default for BearerAuthOptions {
    fn default() -> Self {
        Self {
            required_scopes: Vec::new(),
            resource_metadata_url: None,
        }
    }
}

impl BearerAuthOptions {
    /// Create new options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add required scopes.
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.required_scopes = scopes;
        self
    }

    /// Set the resource metadata URL.
    pub fn with_resource_metadata_url(mut self, url: impl Into<String>) -> Self {
        self.resource_metadata_url = Some(url.into());
        self
    }
}

/// Layer for bearer authentication.
#[derive(Clone)]
pub struct BearerAuthLayer<V> {
    verifier: Arc<V>,
    options: BearerAuthOptions,
}

impl<V> BearerAuthLayer<V>
where
    V: OAuthTokenVerifier + 'static,
{
    /// Create a new bearer auth layer.
    pub fn new(verifier: Arc<V>) -> Self {
        Self {
            verifier,
            options: BearerAuthOptions::default(),
        }
    }

    /// Create a new bearer auth layer with options.
    pub fn with_options(verifier: Arc<V>, options: BearerAuthOptions) -> Self {
        Self { verifier, options }
    }
}

impl<S, V> Layer<S> for BearerAuthLayer<V>
where
    V: OAuthTokenVerifier + 'static,
{
    type Service = BearerAuthMiddleware<S, V>;

    fn layer(&self, inner: S) -> Self::Service {
        BearerAuthMiddleware {
            inner,
            verifier: Arc::clone(&self.verifier),
            options: self.options.clone(),
        }
    }
}

/// Middleware for bearer authentication.
#[derive(Clone)]
pub struct BearerAuthMiddleware<S, V> {
    inner: S,
    verifier: Arc<V>,
    options: BearerAuthOptions,
}

impl<S, V, ReqBody> Service<Request<ReqBody>> for BearerAuthMiddleware<S, V>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
    V: OAuthTokenVerifier + 'static,
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let verifier = Arc::clone(&self.verifier);
        let options = self.options.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Get Authorization header
            let auth_header = req.headers().get(header::AUTHORIZATION);

            let auth_header = match auth_header {
                Some(value) => match value.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return Ok(error_response(
                            StatusCode::UNAUTHORIZED,
                            "invalid_token",
                            "Invalid Authorization header encoding",
                            &options,
                        ));
                    }
                },
                None => {
                    return Ok(error_response(
                        StatusCode::UNAUTHORIZED,
                        "invalid_token",
                        "Missing Authorization header",
                        &options,
                    ));
                }
            };

            // Parse Bearer token
            let token = match parse_bearer_token(auth_header) {
                Some(t) => t,
                None => {
                    return Ok(error_response(
                        StatusCode::UNAUTHORIZED,
                        "invalid_token",
                        "Invalid Authorization header format, expected 'Bearer TOKEN'",
                        &options,
                    ));
                }
            };

            // Verify the token
            let auth_info = match verifier.verify_access_token(token).await {
                Ok(info) => info,
                Err(e) => {
                    let (error, description) = match e {
                        OAuthProviderError::InvalidToken(msg) => ("invalid_token", msg),
                        _ => ("server_error", e.to_string()),
                    };
                    return Ok(error_response(
                        StatusCode::UNAUTHORIZED,
                        error,
                        &description,
                        &options,
                    ));
                }
            };

            // Check expiration
            if auth_info.is_expired() {
                return Ok(error_response(
                    StatusCode::UNAUTHORIZED,
                    "invalid_token",
                    "Token has expired",
                    &options,
                ));
            }

            // Check required scopes
            if !options.required_scopes.is_empty() {
                let scope_refs: Vec<&str> = options.required_scopes.iter().map(|s| s.as_str()).collect();
                if !auth_info.has_scopes(&scope_refs) {
                    return Ok(error_response(
                        StatusCode::FORBIDDEN,
                        "insufficient_scope",
                        "Insufficient scope",
                        &options,
                    ));
                }
            }

            // Add auth info to request extensions
            req.extensions_mut().insert(auth_info);

            // Continue with the request
            inner.call(req).await
        })
    }
}

/// Parse a Bearer token from the Authorization header.
fn parse_bearer_token(header: &str) -> Option<&str> {
    let parts: Vec<&str> = header.splitn(2, ' ').collect();
    if parts.len() == 2 && parts[0].eq_ignore_ascii_case("bearer") {
        Some(parts[1])
    } else {
        None
    }
}

/// Create an error response with WWW-Authenticate header.
fn error_response(
    status: StatusCode,
    error: &str,
    description: &str,
    options: &BearerAuthOptions,
) -> Response<Body> {
    let mut www_auth = format!("Bearer error=\"{}\", error_description=\"{}\"", error, description);

    if !options.required_scopes.is_empty() {
        www_auth.push_str(&format!(", scope=\"{}\"", options.required_scopes.join(" ")));
    }

    if let Some(ref url) = options.resource_metadata_url {
        www_auth.push_str(&format!(", resource_metadata=\"{}\"", url));
    }

    let body = OAuthErrorResponse {
        error: error.to_string(),
        error_description: Some(description.to_string()),
        error_uri: None,
    };

    let json_body = serde_json::to_string(&body).unwrap_or_default();

    Response::builder()
        .status(status)
        .header(header::WWW_AUTHENTICATE, www_auth)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body))
        .unwrap()
}

/// Extension trait for extracting auth info from requests.
#[allow(dead_code)]
pub trait AuthInfoExt {
    /// Get the auth info from the request extensions.
    fn auth_info(&self) -> Option<&AuthInfo>;
}

impl<B> AuthInfoExt for Request<B> {
    fn auth_info(&self) -> Option<&AuthInfo> {
        self.extensions().get::<AuthInfo>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bearer_token() {
        assert_eq!(parse_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(parse_bearer_token("bearer xyz789"), Some("xyz789"));
        assert_eq!(parse_bearer_token("Basic abc123"), None);
        assert_eq!(parse_bearer_token("Bearer"), None);
        assert_eq!(parse_bearer_token(""), None);
    }
}
