//! DNS rebinding protection middleware.
//!
//! This module provides middleware to protect against DNS rebinding attacks by validating
//! the Host header against an allowed list of hostnames.
//!
//! DNS rebinding attacks can bypass same-origin policy by manipulating DNS to point a domain
//! to a localhost address, allowing malicious websites to access your local server.
//!
//! ## Example
//!
//! ```ignore
//! use mcp_server::http::dns_protection::{host_header_validation, localhost_host_validation};
//!
//! // Custom allowed hostnames
//! let layer = host_header_validation(vec!["localhost", "127.0.0.1", "[::1]"]);
//!
//! // Or use the convenience function for localhost-only
//! let layer = localhost_host_validation();
//! ```

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{header, Request, Response, StatusCode};
use tower::{Layer, Service};

/// JSON-RPC error response for DNS rebinding protection failures.
fn json_rpc_error_response(message: &str) -> Response<Body> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32000,
            "message": message
        },
        "id": null
    });

    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// Extract hostname from Host header, ignoring port.
///
/// Handles IPv4, IPv6 (with brackets), and regular hostnames.
fn extract_hostname(host_header: &str) -> Option<String> {
    // Try to parse as a URL to extract hostname
    // We prepend "http://" to make it a valid URL
    let url_str = format!("http://{}", host_header);
    match url::Url::parse(&url_str) {
        Ok(url) => url.host_str().map(|s| s.to_string()),
        Err(_) => None,
    }
}

/// Configuration for DNS rebinding protection.
#[derive(Debug, Clone)]
pub struct DnsProtectionConfig {
    /// Set of allowed hostnames (without ports).
    /// For IPv6, include the address with brackets (e.g., "[::1]").
    pub allowed_hostnames: HashSet<String>,
}

impl DnsProtectionConfig {
    /// Create a new DNS protection configuration with the given allowed hostnames.
    pub fn new<I, S>(hostnames: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed_hostnames: hostnames.into_iter().map(|s| s.into()).collect(),
        }
    }

    /// Create a configuration that only allows localhost connections.
    pub fn localhost() -> Self {
        Self::new(["localhost", "127.0.0.1", "[::1]"])
    }

    /// Check if the hostname is allowed.
    pub fn is_allowed(&self, hostname: &str) -> bool {
        self.allowed_hostnames.contains(hostname)
    }
}

impl Default for DnsProtectionConfig {
    fn default() -> Self {
        Self::localhost()
    }
}

/// Layer for DNS rebinding protection.
#[derive(Debug, Clone)]
pub struct DnsProtectionLayer {
    config: Arc<DnsProtectionConfig>,
}

impl DnsProtectionLayer {
    /// Create a new DNS protection layer with the given configuration.
    pub fn new(config: DnsProtectionConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Create a layer that only allows localhost connections.
    pub fn localhost() -> Self {
        Self::new(DnsProtectionConfig::localhost())
    }
}

impl<S> Layer<S> for DnsProtectionLayer {
    type Service = DnsProtectionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DnsProtectionService {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Service for DNS rebinding protection.
#[derive(Debug, Clone)]
pub struct DnsProtectionService<S> {
    inner: S,
    config: Arc<DnsProtectionConfig>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for DnsProtectionService<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Get Host header
            let host_header = req.headers().get(header::HOST);

            let host_header = match host_header {
                Some(value) => match value.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return Ok(json_rpc_error_response("Invalid Host header encoding"));
                    }
                },
                None => {
                    return Ok(json_rpc_error_response("Missing Host header"));
                }
            };

            // Extract hostname (ignoring port)
            let hostname = match extract_hostname(host_header) {
                Some(h) => h,
                None => {
                    return Ok(json_rpc_error_response(&format!(
                        "Invalid Host header: {}",
                        host_header
                    )));
                }
            };

            // Validate hostname
            if !config.is_allowed(&hostname) {
                return Ok(json_rpc_error_response(&format!(
                    "Invalid Host: {}",
                    hostname
                )));
            }

            // Host is valid, proceed with the request
            inner.call(req).await
        })
    }
}

/// Create a DNS rebinding protection layer with custom allowed hostnames.
///
/// # Arguments
///
/// * `hostnames` - List of allowed hostnames (without ports).
///   For IPv6, provide the address with brackets (e.g., "[::1]").
///
/// # Example
///
/// ```ignore
/// use mcp_server::http::dns_protection::host_header_validation;
///
/// let layer = host_header_validation(vec!["localhost", "127.0.0.1", "example.com"]);
/// ```
pub fn host_header_validation<I, S>(hostnames: I) -> DnsProtectionLayer
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    DnsProtectionLayer::new(DnsProtectionConfig::new(hostnames))
}

/// Create a DNS rebinding protection layer for localhost only.
///
/// Allows only localhost, 127.0.0.1, and [::1] (IPv6 localhost) hostnames.
///
/// # Example
///
/// ```ignore
/// use mcp_server::http::dns_protection::localhost_host_validation;
///
/// let layer = localhost_host_validation();
/// ```
pub fn localhost_host_validation() -> DnsProtectionLayer {
    DnsProtectionLayer::localhost()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hostname_simple() {
        assert_eq!(extract_hostname("localhost"), Some("localhost".to_string()));
        assert_eq!(
            extract_hostname("localhost:3000"),
            Some("localhost".to_string())
        );
    }

    #[test]
    fn test_extract_hostname_ipv4() {
        assert_eq!(
            extract_hostname("127.0.0.1"),
            Some("127.0.0.1".to_string())
        );
        assert_eq!(
            extract_hostname("127.0.0.1:8080"),
            Some("127.0.0.1".to_string())
        );
    }

    #[test]
    fn test_extract_hostname_ipv6() {
        assert_eq!(extract_hostname("[::1]"), Some("[::1]".to_string()));
        assert_eq!(extract_hostname("[::1]:8080"), Some("[::1]".to_string()));
    }

    #[test]
    fn test_extract_hostname_domain() {
        assert_eq!(
            extract_hostname("example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_hostname("example.com:443"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_hostname("api.example.com:8080"),
            Some("api.example.com".to_string())
        );
    }

    #[test]
    fn test_config_localhost() {
        let config = DnsProtectionConfig::localhost();
        assert!(config.is_allowed("localhost"));
        assert!(config.is_allowed("127.0.0.1"));
        assert!(config.is_allowed("[::1]"));
        assert!(!config.is_allowed("example.com"));
    }

    #[test]
    fn test_config_custom() {
        let config = DnsProtectionConfig::new(["localhost", "example.com"]);
        assert!(config.is_allowed("localhost"));
        assert!(config.is_allowed("example.com"));
        assert!(!config.is_allowed("127.0.0.1"));
        assert!(!config.is_allowed("other.com"));
    }
}
