//! Token revocation endpoint handler (RFC 7009).

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Form;
use serde::Deserialize;

use mcp_core::auth::{OAuthErrorResponse, OAuthTokenRevocationRequest};

use crate::auth::provider::OAuthServerProvider;
use crate::auth::router::OAuthRouterState;

/// Token revocation request parameters.
#[derive(Debug, Deserialize)]
pub struct RevokeRequest {
    /// The token to revoke.
    pub token: String,
    /// A hint about the type of the token.
    pub token_type_hint: Option<String>,
    /// Client ID.
    pub client_id: Option<String>,
    /// Client secret.
    pub client_secret: Option<String>,
}

/// Token revocation endpoint handler.
pub async fn revoke_handler<P: OAuthServerProvider + 'static>(
    State(state): State<Arc<OAuthRouterState<P>>>,
    Form(request): Form<RevokeRequest>,
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

    // Revoke the token
    let revocation_request = OAuthTokenRevocationRequest {
        token: request.token,
        token_type_hint: request.token_type_hint,
    };

    match state.provider.revoke_token(&client, revocation_request).await {
        Ok(()) => {
            // RFC 7009: Return 200 OK with empty body on success
            StatusCode::OK.into_response()
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

/// Create an error response.
fn error_response(status: StatusCode, error: &str, description: &str) -> Response {
    let body = OAuthErrorResponse {
        error: error.to_string(),
        error_description: Some(description.to_string()),
        error_uri: None,
    };

    (status, axum::Json(body)).into_response()
}
