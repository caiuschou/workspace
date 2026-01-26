//! Authorization endpoint handler.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};

use mcp_core::auth::AuthorizationParams;

use crate::auth::provider::{AuthorizeResponse, OAuthServerProvider};
use crate::auth::router::OAuthRouterState;

/// Query parameters for the authorization endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct AuthorizeQuery {
    /// Response type (must be "code").
    pub response_type: String,
    /// Client ID.
    pub client_id: String,
    /// Redirect URI.
    pub redirect_uri: String,
    /// State parameter for CSRF protection.
    pub state: Option<String>,
    /// Requested scope.
    pub scope: Option<String>,
    /// PKCE code challenge.
    pub code_challenge: Option<String>,
    /// PKCE code challenge method (must be "S256").
    pub code_challenge_method: Option<String>,
    /// Resource indicator (RFC 8707).
    pub resource: Option<String>,
}

/// Authorization endpoint handler.
pub async fn authorize_handler<P: OAuthServerProvider + 'static>(
    State(state): State<Arc<OAuthRouterState<P>>>,
    Query(query): Query<AuthorizeQuery>,
) -> Response {
    // Validate response_type
    if query.response_type != "code" {
        return error_redirect(
            &query.redirect_uri,
            "unsupported_response_type",
            Some("Only 'code' response type is supported"),
            query.state.as_deref(),
        );
    }

    // Validate code_challenge
    let code_challenge = match query.code_challenge {
        Some(challenge) => challenge,
        None => {
            return error_redirect(
                &query.redirect_uri,
                "invalid_request",
                Some("code_challenge is required"),
                query.state.as_deref(),
            );
        }
    };

    // Validate code_challenge_method
    if let Some(ref method) = query.code_challenge_method {
        if method != "S256" {
            return error_redirect(
                &query.redirect_uri,
                "invalid_request",
                Some("Only 'S256' code challenge method is supported"),
                query.state.as_deref(),
            );
        }
    }

    // Get client
    let client = match state.provider.clients_store().get_client(&query.client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => {
            return error_redirect(
                &query.redirect_uri,
                "invalid_client",
                Some("Unknown client"),
                query.state.as_deref(),
            );
        }
        Err(e) => {
            return error_redirect(
                &query.redirect_uri,
                "server_error",
                Some(&e.to_string()),
                query.state.as_deref(),
            );
        }
    };

    // Validate redirect_uri
    if !client.metadata.redirect_uris.contains(&query.redirect_uri) {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid redirect_uri",
        ).into_response();
    }

    // Parse scopes
    let scopes = query.scope.map(|s| {
        s.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>()
    });

    let params = AuthorizationParams {
        state: query.state.clone(),
        scopes,
        code_challenge,
        redirect_uri: query.redirect_uri.clone(),
        resource: query.resource,
    };

    // Call provider
    match state.provider.authorize(&client, params).await {
        Ok(AuthorizeResponse::Redirect { url }) => {
            Redirect::to(&url).into_response()
        }
        Ok(AuthorizeResponse::Html { content }) => {
            Html(content).into_response()
        }
        Ok(AuthorizeResponse::Error { error, description }) => {
            error_redirect(
                &query.redirect_uri,
                &error,
                description.as_deref(),
                query.state.as_deref(),
            )
        }
        Err(e) => {
            error_redirect(
                &query.redirect_uri,
                e.error_code(),
                Some(&e.to_string()),
                query.state.as_deref(),
            )
        }
    }
}

/// Create an error redirect response.
fn error_redirect(
    redirect_uri: &str,
    error: &str,
    description: Option<&str>,
    state: Option<&str>,
) -> Response {
    let mut url = match url::Url::parse(redirect_uri) {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid redirect_uri").into_response();
        }
    };

    url.query_pairs_mut().append_pair("error", error);

    if let Some(desc) = description {
        url.query_pairs_mut().append_pair("error_description", desc);
    }

    if let Some(s) = state {
        url.query_pairs_mut().append_pair("state", s);
    }

    Redirect::to(url.as_str()).into_response()
}
