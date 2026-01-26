//! Token endpoint handler.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Form;
use serde::Deserialize;

use mcp_core::auth::OAuthErrorResponse;

use crate::auth::provider::OAuthServerProvider;
use crate::auth::router::OAuthRouterState;

/// Token request parameters.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// Grant type.
    pub grant_type: String,
    /// Authorization code (for authorization_code grant).
    pub code: Option<String>,
    /// Redirect URI (for authorization_code grant).
    pub redirect_uri: Option<String>,
    /// Client ID.
    pub client_id: Option<String>,
    /// Client secret.
    pub client_secret: Option<String>,
    /// PKCE code verifier.
    pub code_verifier: Option<String>,
    /// Refresh token (for refresh_token grant).
    pub refresh_token: Option<String>,
    /// Requested scope.
    pub scope: Option<String>,
    /// Resource indicator (RFC 8707).
    pub resource: Option<String>,
}

/// Token endpoint handler.
pub async fn token_handler<P: OAuthServerProvider + 'static>(
    State(state): State<Arc<OAuthRouterState<P>>>,
    Form(request): Form<TokenRequest>,
) -> Response {
    // Get client ID
    let client_id = match &request.client_id {
        Some(id) => id.clone(),
        None => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "client_id is required",
            );
        }
    };

    // Get client
    let client = match state.provider.clients_store().get_client(&client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "invalid_client",
                "Unknown client",
            );
        }
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                &e.to_string(),
            );
        }
    };

    // Validate client secret if the client has one
    if let Some(expected_secret) = &client.client_info.client_secret {
        match &request.client_secret {
            Some(secret) if secret == expected_secret => {}
            _ => {
                return error_response(
                    StatusCode::UNAUTHORIZED,
                    "invalid_client",
                    "Invalid client credentials",
                );
            }
        }
    }

    // Handle grant type
    match request.grant_type.as_str() {
        "authorization_code" => {
            handle_authorization_code_grant(&state, &client, &request).await
        }
        "refresh_token" => {
            handle_refresh_token_grant(&state, &client, &request).await
        }
        _ => {
            error_response(
                StatusCode::BAD_REQUEST,
                "unsupported_grant_type",
                &format!("Unsupported grant type: {}", request.grant_type),
            )
        }
    }
}

/// Handle authorization code grant.
async fn handle_authorization_code_grant<P: OAuthServerProvider + 'static>(
    state: &OAuthRouterState<P>,
    client: &mcp_core::auth::OAuthClientInformationFull,
    request: &TokenRequest,
) -> Response {
    let code = match &request.code {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "code is required for authorization_code grant",
            );
        }
    };

    // PKCE validation (if not skipped)
    if !state.provider.skip_local_pkce_validation() {
        let code_verifier = match &request.code_verifier {
            Some(v) => v,
            None => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "code_verifier is required",
                );
            }
        };

        // Get the expected challenge
        let expected_challenge = match state.provider.challenge_for_authorization_code(client, code).await {
            Ok(c) => c,
            Err(e) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    e.error_code(),
                    &e.to_string(),
                );
            }
        };

        // Verify PKCE
        if !verify_pkce_challenge(code_verifier, &expected_challenge) {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_grant",
                "Invalid code_verifier",
            );
        }
    }

    // Exchange the code for tokens
    match state.provider.exchange_authorization_code(
        client,
        code,
        request.code_verifier.as_deref(),
        request.redirect_uri.as_deref(),
        request.resource.as_deref(),
    ).await {
        Ok(tokens) => {
            (StatusCode::OK, axum::Json(tokens)).into_response()
        }
        Err(e) => {
            error_response(
                StatusCode::BAD_REQUEST,
                e.error_code(),
                &e.to_string(),
            )
        }
    }
}

/// Handle refresh token grant.
async fn handle_refresh_token_grant<P: OAuthServerProvider + 'static>(
    state: &OAuthRouterState<P>,
    client: &mcp_core::auth::OAuthClientInformationFull,
    request: &TokenRequest,
) -> Response {
    let refresh_token = match &request.refresh_token {
        Some(t) => t,
        None => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "refresh_token is required for refresh_token grant",
            );
        }
    };

    // Parse scopes
    let scopes = request.scope.as_ref().map(|s| {
        s.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>()
    });

    // Exchange the refresh token
    match state.provider.exchange_refresh_token(
        client,
        refresh_token,
        scopes.as_deref(),
        request.resource.as_deref(),
    ).await {
        Ok(tokens) => {
            (StatusCode::OK, axum::Json(tokens)).into_response()
        }
        Err(e) => {
            error_response(
                StatusCode::BAD_REQUEST,
                e.error_code(),
                &e.to_string(),
            )
        }
    }
}

/// Verify a PKCE code challenge.
fn verify_pkce_challenge(verifier: &str, challenge: &str) -> bool {
    use sha2::{Sha256, Digest};
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let computed = URL_SAFE_NO_PAD.encode(hash);

    computed == challenge
}

/// Create an error response.
fn error_response(status: StatusCode, error: &str, description: &str) -> Response {
    let body = OAuthErrorResponse {
        error: error.to_string(),
        error_description: Some(description.to_string()),
        error_uri: None,
    };

    (status, axum::Json(body)).into_response()
}
