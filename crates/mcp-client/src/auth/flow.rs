//! OAuth authorization flow implementation.
//!
//! This module provides the main OAuth authorization flow functions.

use mcp_core::auth::{
    OAuthClientInformation, OAuthClientInformationFull, OAuthClientMetadata, OAuthMetadata,
    OAuthTokens,
};

use super::discovery::{discover_authorization_server_metadata, discover_protected_resource_metadata};
use super::provider::{AuthResult, InvalidationScope, OAuthClientError, OAuthClientProvider};

/// Main entry point for the OAuth authorization flow.
///
/// This function orchestrates the full OAuth flow:
/// 1. Discovers authorization server metadata
/// 2. Handles client registration if needed
/// 3. Attempts to refresh existing tokens
/// 4. Initiates new authorization if needed
pub async fn auth<P: OAuthClientProvider>(
    provider: &P,
    options: AuthOptions<'_>,
) -> Result<AuthResult, OAuthClientError> {
    match auth_internal(provider, &options).await {
        Ok(result) => Ok(result),
        Err(OAuthClientError::InvalidClient(_)) | Err(OAuthClientError::Unauthorized(_)) => {
            // Invalidate credentials and retry
            provider.invalidate_credentials(InvalidationScope::All).await?;
            auth_internal(provider, &options).await
        }
        Err(OAuthClientError::InvalidGrant(_)) => {
            // Invalidate tokens and retry
            provider.invalidate_credentials(InvalidationScope::Tokens).await?;
            auth_internal(provider, &options).await
        }
        Err(e) => Err(e),
    }
}

/// Options for the auth flow.
pub struct AuthOptions<'a> {
    /// The MCP server URL.
    pub server_url: &'a str,
    /// Authorization code (if completing a redirect flow).
    pub authorization_code: Option<&'a str>,
    /// Requested scope.
    pub scope: Option<&'a str>,
    /// Resource metadata URL (from WWW-Authenticate header).
    pub resource_metadata_url: Option<&'a str>,
}

impl<'a> AuthOptions<'a> {
    /// Create new auth options.
    pub fn new(server_url: &'a str) -> Self {
        Self {
            server_url,
            authorization_code: None,
            scope: None,
            resource_metadata_url: None,
        }
    }

    /// Set the authorization code.
    pub fn with_authorization_code(mut self, code: &'a str) -> Self {
        self.authorization_code = Some(code);
        self
    }

    /// Set the scope.
    pub fn with_scope(mut self, scope: &'a str) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set the resource metadata URL.
    pub fn with_resource_metadata_url(mut self, url: &'a str) -> Self {
        self.resource_metadata_url = Some(url);
        self
    }
}

/// Internal auth implementation.
async fn auth_internal<P: OAuthClientProvider>(
    provider: &P,
    options: &AuthOptions<'_>,
) -> Result<AuthResult, OAuthClientError> {
    // Try to discover resource metadata
    let resource_metadata = discover_protected_resource_metadata(
        options.server_url,
        options.resource_metadata_url,
    ).ok();

    // Determine authorization server URL
    let auth_server_url = resource_metadata
        .as_ref()
        .and_then(|m| m.authorization_servers.as_ref())
        .and_then(|servers| servers.first())
        .map(|s| s.as_str())
        .unwrap_or(options.server_url);

    // Discover authorization server metadata
    let metadata = discover_authorization_server_metadata(auth_server_url)?;

    // Get or register client
    let client_info = match provider.client_information().await {
        Some(info) => info,
        None => {
            if options.authorization_code.is_some() {
                return Err(OAuthClientError::InvalidRequest(
                    "Client information required for authorization code exchange".to_string(),
                ));
            }

            // Register the client
            let full_info = register_client(&metadata, provider.client_metadata())?;
            provider.save_client_information(full_info.client_info.clone()).await?;
            full_info.client_info
        }
    };

    // Select resource URL
    let resource = provider
        .validate_resource_url(options.server_url, resource_metadata.as_ref().map(|m| m.resource.as_str()))
        .await?;

    // Non-interactive flows
    if provider.redirect_url().is_none() {
        let tokens = fetch_token(provider, &metadata, &client_info, resource.as_deref(), options.authorization_code).await?;
        provider.save_tokens(tokens).await?;
        return Ok(AuthResult::Authorized);
    }

    // Exchange authorization code if provided
    if let Some(code) = options.authorization_code {
        let tokens = exchange_authorization_code(
            provider,
            &metadata,
            &client_info,
            code,
            resource.as_deref(),
        ).await?;
        provider.save_tokens(tokens).await?;
        return Ok(AuthResult::Authorized);
    }

    // Try to refresh existing tokens
    if let Some(tokens) = provider.tokens().await {
        if let Some(refresh_token) = &tokens.refresh_token {
            match refresh_authorization(&metadata, &client_info, refresh_token, resource.as_deref()).await {
                Ok(new_tokens) => {
                    provider.save_tokens(new_tokens).await?;
                    return Ok(AuthResult::Authorized);
                }
                Err(_) => {
                    // Refresh failed, continue to new authorization
                }
            }
        }
    }

    // Start new authorization
    let state = provider.state().await;
    let scope_from_metadata = resource_metadata
        .as_ref()
        .and_then(|m| m.scopes_supported.as_ref())
        .map(|s| s.join(" "));
    let scope = options.scope.map(|s| s.to_string()).or(scope_from_metadata);
    let (auth_url, code_verifier) = start_authorization(
        &metadata,
        &client_info,
        provider.redirect_url().unwrap(),
        scope.as_deref(),
        state.as_deref(),
        resource.as_deref(),
    )?;

    provider.save_code_verifier(code_verifier).await?;
    provider.redirect_to_authorization(&auth_url).await?;

    Ok(AuthResult::Redirect)
}

/// Register a client with the authorization server (RFC 7591).
pub fn register_client(
    metadata: &OAuthMetadata,
    client_metadata: &OAuthClientMetadata,
) -> Result<OAuthClientInformationFull, OAuthClientError> {
    let registration_endpoint = metadata.registration_endpoint.as_ref()
        .ok_or_else(|| OAuthClientError::Server(
            "Authorization server does not support dynamic client registration".to_string(),
        ))?;

    let response = ureq::post(registration_endpoint)
        .set("Content-Type", "application/json")
        .send_json(client_metadata)
        .map_err(|e| OAuthClientError::Network(format!("Registration failed: {}", e)))?;

    let full_info: OAuthClientInformationFull = response
        .into_json()
        .map_err(|e| OAuthClientError::Server(format!("Invalid registration response: {}", e)))?;

    Ok(full_info)
}

/// Start the authorization flow (RFC 6749 + PKCE).
pub fn start_authorization(
    metadata: &OAuthMetadata,
    client_info: &OAuthClientInformation,
    redirect_url: &str,
    scope: Option<&str>,
    state: Option<&str>,
    resource: Option<&str>,
) -> Result<(String, String), OAuthClientError> {
    // Generate PKCE challenge
    let (code_verifier, code_challenge) = generate_pkce_challenge();

    // Build authorization URL
    let mut url = url::Url::parse(&metadata.authorization_endpoint)
        .map_err(|e| OAuthClientError::InvalidRequest(format!("Invalid authorization endpoint: {}", e)))?;

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &client_info.client_id)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("redirect_uri", redirect_url);

    if let Some(state) = state {
        url.query_pairs_mut().append_pair("state", state);
    }

    if let Some(scope) = scope {
        url.query_pairs_mut().append_pair("scope", scope);
    }

    if let Some(resource) = resource {
        url.query_pairs_mut().append_pair("resource", resource);
    }

    Ok((url.to_string(), code_verifier))
}

/// Exchange authorization code for tokens.
async fn exchange_authorization_code<P: OAuthClientProvider>(
    provider: &P,
    metadata: &OAuthMetadata,
    client_info: &OAuthClientInformation,
    code: &str,
    resource: Option<&str>,
) -> Result<OAuthTokens, OAuthClientError> {
    let code_verifier = provider.code_verifier().await?;
    let redirect_url = provider.redirect_url()
        .ok_or_else(|| OAuthClientError::InvalidRequest("redirect_url required".to_string()))?;

    let mut params = vec![
        ("grant_type".to_string(), "authorization_code".to_string()),
        ("code".to_string(), code.to_string()),
        ("code_verifier".to_string(), code_verifier),
        ("redirect_uri".to_string(), redirect_url.to_string()),
        ("client_id".to_string(), client_info.client_id.clone()),
    ];

    if let Some(secret) = &client_info.client_secret {
        params.push(("client_secret".to_string(), secret.clone()));
    }

    if let Some(resource) = resource {
        params.push(("resource".to_string(), resource.to_string()));
    }

    execute_token_request(&metadata.token_endpoint, params).await
}

/// Refresh authorization tokens.
async fn refresh_authorization(
    metadata: &OAuthMetadata,
    client_info: &OAuthClientInformation,
    refresh_token: &str,
    resource: Option<&str>,
) -> Result<OAuthTokens, OAuthClientError> {
    let mut params = vec![
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh_token.to_string()),
        ("client_id".to_string(), client_info.client_id.clone()),
    ];

    if let Some(secret) = &client_info.client_secret {
        params.push(("client_secret".to_string(), secret.clone()));
    }

    if let Some(resource) = resource {
        params.push(("resource".to_string(), resource.to_string()));
    }

    let mut tokens = execute_token_request(&metadata.token_endpoint, params).await?;

    // Preserve original refresh token if not replaced
    if tokens.refresh_token.is_none() {
        tokens.refresh_token = Some(refresh_token.to_string());
    }

    Ok(tokens)
}

/// Fetch tokens using provider's custom grant or authorization code.
async fn fetch_token<P: OAuthClientProvider>(
    provider: &P,
    metadata: &OAuthMetadata,
    client_info: &OAuthClientInformation,
    resource: Option<&str>,
    authorization_code: Option<&str>,
) -> Result<OAuthTokens, OAuthClientError> {
    let scope = provider.client_metadata().scope.as_deref();

    // Check for custom grant parameters
    if let Some(params) = provider.prepare_token_request(scope).await {
        let mut all_params = params;
        all_params.push(("client_id".to_string(), client_info.client_id.clone()));

        if let Some(secret) = &client_info.client_secret {
            all_params.push(("client_secret".to_string(), secret.clone()));
        }

        if let Some(resource) = resource {
            all_params.push(("resource".to_string(), resource.to_string()));
        }

        return execute_token_request(&metadata.token_endpoint, all_params).await;
    }

    // Default to authorization code flow
    if let Some(code) = authorization_code {
        return exchange_authorization_code(provider, metadata, client_info, code, resource).await;
    }

    Err(OAuthClientError::InvalidRequest(
        "Either custom grant or authorization code required".to_string(),
    ))
}

/// Execute a token request.
async fn execute_token_request(
    token_endpoint: &str,
    params: Vec<(String, String)>,
) -> Result<OAuthTokens, OAuthClientError> {
    let body: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    let response = ureq::post(token_endpoint)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("Accept", "application/json")
        .send_form(&body)
        .map_err(|e| {
            // Try to parse OAuth error
            OAuthClientError::Network(format!("Token request failed: {}", e))
        })?;

    let tokens: OAuthTokens = response
        .into_json()
        .map_err(|e| OAuthClientError::Server(format!("Invalid token response: {}", e)))?;

    Ok(tokens)
}

/// Generate PKCE code verifier and challenge.
fn generate_pkce_challenge() -> (String, String) {
    use sha2::{Sha256, Digest};
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

    // Generate random verifier (43-128 characters)
    let verifier: String = (0..64)
        .map(|_| {
            let idx = rand_byte() % 66;
            PKCE_CHARSET[idx as usize] as char
        })
        .collect();

    // Create S256 challenge
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let challenge = URL_SAFE_NO_PAD.encode(hash);

    (verifier, challenge)
}

/// Get a pseudo-random byte.
fn rand_byte() -> u8 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Simple PRNG based on time - for production, use a proper CSPRNG
    let seed = duration.as_nanos() as u64;
    ((seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) >> 56) as u8
}

/// Characters allowed in PKCE code verifier.
const PKCE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
