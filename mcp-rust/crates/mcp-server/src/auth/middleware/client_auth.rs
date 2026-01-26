//! Client authentication middleware.
//!
//! This middleware validates OAuth client credentials and adds
//! client information to the request extensions.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{header, Request, Response, StatusCode};
use tower::{Layer, Service};

use mcp_core::auth::{OAuthClientInformationFull, OAuthErrorResponse};

use crate::auth::clients::OAuthRegisteredClientsStore;

/// Layer for client authentication.
#[derive(Clone)]
pub struct ClientAuthLayer<S> {
    store: Arc<S>,
}

impl<S> ClientAuthLayer<S>
where
    S: OAuthRegisteredClientsStore + 'static,
{
    /// Create a new client auth layer.
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }
}

impl<Inner, S> Layer<Inner> for ClientAuthLayer<S>
where
    S: OAuthRegisteredClientsStore + 'static,
{
    type Service = ClientAuthMiddleware<Inner, S>;

    fn layer(&self, inner: Inner) -> Self::Service {
        ClientAuthMiddleware {
            inner,
            store: Arc::clone(&self.store),
        }
    }
}

/// Middleware for client authentication.
#[derive(Clone)]
pub struct ClientAuthMiddleware<Inner, S> {
    inner: Inner,
    store: Arc<S>,
}

impl<Inner, S, ReqBody> Service<Request<ReqBody>> for ClientAuthMiddleware<Inner, S>
where
    Inner: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    Inner::Future: Send,
    S: OAuthRegisteredClientsStore + 'static,
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Inner::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let store = Arc::clone(&self.store);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Try to extract client credentials from Authorization header (Basic auth)
            // or from request body (if this is a form POST)
            let (client_id, client_secret) = match extract_client_credentials(&req) {
                Some(creds) => creds,
                None => {
                    return Ok(error_response(
                        StatusCode::UNAUTHORIZED,
                        "invalid_client",
                        "Missing client credentials",
                    ));
                }
            };

            // Get client from store
            let client = match store.get_client(&client_id).await {
                Ok(Some(client)) => client,
                Ok(None) => {
                    return Ok(error_response(
                        StatusCode::UNAUTHORIZED,
                        "invalid_client",
                        "Unknown client",
                    ));
                }
                Err(e) => {
                    return Ok(error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "server_error",
                        &e.to_string(),
                    ));
                }
            };

            // Validate client secret if the client has one
            if let Some(expected_secret) = &client.client_info.client_secret {
                match &client_secret {
                    Some(secret) if secret == expected_secret => {}
                    _ => {
                        return Ok(error_response(
                            StatusCode::UNAUTHORIZED,
                            "invalid_client",
                            "Invalid client credentials",
                        ));
                    }
                }
            }

            // Add client info to request extensions
            req.extensions_mut().insert(client);

            // Continue with the request
            inner.call(req).await
        })
    }
}

/// Extract client credentials from the request.
///
/// Tries Basic auth first, then falls back to looking for client_id in query params.
fn extract_client_credentials<B>(req: &Request<B>) -> Option<(String, Option<String>)> {
    // Try Authorization header (Basic auth)
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(credentials) = parse_basic_auth(auth_str) {
                return Some(credentials);
            }
        }
    }

    // Try query parameters
    if let Some(query) = req.uri().query() {
        let params: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let client_id = params.iter().find(|(k, _)| k == "client_id").map(|(_, v)| v.clone());
        let client_secret = params.iter().find(|(k, _)| k == "client_secret").map(|(_, v)| v.clone());

        if let Some(id) = client_id {
            return Some((id, client_secret));
        }
    }

    None
}

/// Parse Basic auth credentials.
fn parse_basic_auth(header: &str) -> Option<(String, Option<String>)> {
    let parts: Vec<&str> = header.splitn(2, ' ').collect();
    if parts.len() != 2 || !parts[0].eq_ignore_ascii_case("basic") {
        return None;
    }

    use base64::{Engine, engine::general_purpose::STANDARD};

    let decoded = STANDARD.decode(parts[1]).ok()?;
    let credentials = String::from_utf8(decoded).ok()?;

    let cred_parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if cred_parts.is_empty() {
        return None;
    }

    let client_id = cred_parts[0].to_string();
    let client_secret = cred_parts.get(1).map(|s| s.to_string());

    Some((client_id, client_secret))
}

/// Create an error response.
fn error_response(status: StatusCode, error: &str, description: &str) -> Response<Body> {
    let body = OAuthErrorResponse {
        error: error.to_string(),
        error_description: Some(description.to_string()),
        error_uri: None,
    };

    let json_body = serde_json::to_string(&body).unwrap_or_default();

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body))
        .unwrap()
}

/// Extension trait for extracting client info from requests.
#[allow(dead_code)]
pub trait ClientInfoExt {
    /// Get the client info from the request extensions.
    fn client_info(&self) -> Option<&OAuthClientInformationFull>;
}

impl<B> ClientInfoExt for Request<B> {
    fn client_info(&self) -> Option<&OAuthClientInformationFull> {
        self.extensions().get::<OAuthClientInformationFull>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_auth() {
        // "client:secret" base64 encoded
        let encoded = "Y2xpZW50OnNlY3JldA==";
        let result = parse_basic_auth(&format!("Basic {}", encoded));
        assert_eq!(result, Some(("client".to_string(), Some("secret".to_string()))));

        // "client:" base64 encoded (empty secret)
        let encoded = "Y2xpZW50Og==";
        let result = parse_basic_auth(&format!("Basic {}", encoded));
        assert_eq!(result, Some(("client".to_string(), Some("".to_string()))));

        // Invalid
        assert!(parse_basic_auth("Bearer token").is_none());
    }
}
