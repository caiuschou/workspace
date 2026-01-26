//! OAuth metadata endpoint handlers.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Json;

use mcp_core::auth::OAuthMetadata;

use crate::auth::provider::OAuthServerProvider;
use crate::auth::router::OAuthRouterState;

/// Authorization server metadata endpoint handler (RFC 8414).
pub async fn metadata_handler<P: OAuthServerProvider + 'static>(
    State(state): State<Arc<OAuthRouterState<P>>>,
) -> Json<OAuthMetadata> {
    Json(state.metadata.clone())
}
