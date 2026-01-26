//! Dynamic client registration endpoint handler (RFC 7591).

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use mcp_core::auth::{OAuthClientMetadata, OAuthErrorResponse};

use crate::auth::provider::OAuthServerProvider;
use crate::auth::router::OAuthRouterState;

/// Client registration endpoint handler.
pub async fn register_handler<P: OAuthServerProvider + 'static>(
    State(state): State<Arc<OAuthRouterState<P>>>,
    Json(metadata): Json<OAuthClientMetadata>,
) -> Response {
    // Validate redirect_uris
    if metadata.redirect_uris.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_client_metadata",
            "redirect_uris is required",
        );
    }

    // Validate redirect URIs format
    for uri in &metadata.redirect_uris {
        if url::Url::parse(uri).is_err() {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_client_metadata",
                &format!("Invalid redirect_uri: {}", uri),
            );
        }
    }

    // Register the client
    match state.provider.clients_store().register_client(metadata).await {
        Ok(client) => {
            (StatusCode::CREATED, Json(client)).into_response()
        }
        Err(e) => {
            error_response(
                StatusCode::BAD_REQUEST,
                "invalid_client_metadata",
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

    (status, Json(body)).into_response()
}
