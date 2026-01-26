//! OAuth metadata discovery.
//!
//! This module provides functions for discovering OAuth authorization server
//! and protected resource metadata.

use mcp_core::auth::{OAuthMetadata, OAuthProtectedResourceMetadata};

use super::provider::OAuthClientError;

/// Discover OAuth protected resource metadata (RFC 9728).
///
/// Tries path-aware discovery first, then falls back to root discovery.
pub fn discover_protected_resource_metadata(
    server_url: &str,
    resource_metadata_url: Option<&str>,
) -> Result<OAuthProtectedResourceMetadata, OAuthClientError> {
    // If a specific URL is provided, use it
    if let Some(url) = resource_metadata_url {
        return fetch_protected_resource_metadata(url);
    }

    // Try path-aware discovery
    let parsed = url::Url::parse(server_url)
        .map_err(|e| OAuthClientError::InvalidRequest(format!("Invalid server URL: {}", e)))?;

    let path = parsed.path();

    // Try /.well-known/oauth-protected-resource{path}
    let well_known_url = if path == "/" || path.is_empty() {
        format!(
            "{}://{}/.well-known/oauth-protected-resource",
            parsed.scheme(),
            parsed.host_str().unwrap_or("localhost")
        )
    } else {
        format!(
            "{}://{}/.well-known/oauth-protected-resource{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or("localhost"),
            path.trim_end_matches('/')
        )
    };

    match fetch_protected_resource_metadata(&well_known_url) {
        Ok(metadata) => return Ok(metadata),
        Err(_) if path != "/" && !path.is_empty() => {
            // Fall back to root discovery
            let root_url = format!(
                "{}://{}/.well-known/oauth-protected-resource",
                parsed.scheme(),
                parsed.host_str().unwrap_or("localhost")
            );
            return fetch_protected_resource_metadata(&root_url);
        }
        Err(e) => return Err(e),
    }
}

/// Fetch protected resource metadata from a specific URL.
fn fetch_protected_resource_metadata(url: &str) -> Result<OAuthProtectedResourceMetadata, OAuthClientError> {
    let response = ureq::get(url)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| OAuthClientError::Network(format!("Failed to fetch resource metadata: {}", e)))?;

    let metadata: OAuthProtectedResourceMetadata = response
        .into_json()
        .map_err(|e| OAuthClientError::Server(format!("Invalid resource metadata: {}", e)))?;

    Ok(metadata)
}

/// Discover OAuth authorization server metadata (RFC 8414).
///
/// Supports both OAuth 2.0 and OpenID Connect discovery.
pub fn discover_authorization_server_metadata(
    authorization_server_url: &str,
) -> Result<OAuthMetadata, OAuthClientError> {
    let parsed = url::Url::parse(authorization_server_url)
        .map_err(|e| OAuthClientError::InvalidRequest(format!("Invalid authorization server URL: {}", e)))?;

    let path = parsed.path();
    let host = parsed.host_str().unwrap_or("localhost");
    let scheme = parsed.scheme();

    // Build URLs to try
    let mut urls_to_try = Vec::new();

    if path == "/" || path.is_empty() {
        // Root path
        urls_to_try.push(format!("{}://{}/.well-known/oauth-authorization-server", scheme, host));
        urls_to_try.push(format!("{}://{}/.well-known/openid-configuration", scheme, host));
    } else {
        let clean_path = path.trim_end_matches('/');

        // RFC 8414 style: Insert well-known before the path
        urls_to_try.push(format!("{}://{}/.well-known/oauth-authorization-server{}", scheme, host, clean_path));
        urls_to_try.push(format!("{}://{}/.well-known/openid-configuration{}", scheme, host, clean_path));

        // OIDC Discovery 1.0 style: Append well-known after the path
        urls_to_try.push(format!("{}://{}{}/.well-known/openid-configuration", scheme, host, clean_path));
    }

    // Try each URL
    for url in urls_to_try {
        match fetch_authorization_server_metadata(&url) {
            Ok(metadata) => return Ok(metadata),
            Err(_) => continue,
        }
    }

    Err(OAuthClientError::Server(
        "Could not discover authorization server metadata".to_string(),
    ))
}

/// Fetch authorization server metadata from a specific URL.
fn fetch_authorization_server_metadata(url: &str) -> Result<OAuthMetadata, OAuthClientError> {
    let response = ureq::get(url)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| OAuthClientError::Network(format!("Failed to fetch metadata: {}", e)))?;

    let metadata: OAuthMetadata = response
        .into_json()
        .map_err(|e| OAuthClientError::Server(format!("Invalid metadata: {}", e)))?;

    Ok(metadata)
}

/// Get the OAuth protected resource metadata URL from a server URL.
pub fn get_protected_resource_metadata_url(server_url: &str) -> Result<String, OAuthClientError> {
    let parsed = url::Url::parse(server_url)
        .map_err(|e| OAuthClientError::InvalidRequest(format!("Invalid server URL: {}", e)))?;

    let path = parsed.path();
    let rs_path = if path == "/" || path.is_empty() {
        String::new()
    } else {
        path.trim_end_matches('/').to_string()
    };

    Ok(format!(
        "{}://{}/.well-known/oauth-protected-resource{}",
        parsed.scheme(),
        parsed.host_str().unwrap_or("localhost"),
        rs_path
    ))
}
