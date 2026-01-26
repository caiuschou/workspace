pub mod auth;
pub mod http;
pub mod server;
pub mod websocket;

pub use server::{InMemoryTaskStore, McpServer, Server, ServerError, ServerOptions};

pub use http::{
    BufferedEvent, EventBuffer, EventBufferConfig, HttpResponse, HttpServerError,
    HttpServerHandler, HttpServerOptions, LegacySseConfig, LegacySseState, RequestHeaders,
    SessionConfig, SessionManager, SessionState, SseResponseBuilder, SseWriter,
    generate_session_id,
};

#[cfg(feature = "tokio")]
pub use http::SseBroadcaster;

#[cfg(feature = "axum")]
pub use http::axum_handler::{AxumHandlerConfig, AxumHandlerState, create_router};

#[cfg(feature = "axum")]
pub use http::create_legacy_sse_router;

#[cfg(feature = "axum")]
pub use http::{
    host_header_validation, localhost_host_validation, DnsProtectionConfig, DnsProtectionLayer,
    DnsProtectionService,
};

#[cfg(feature = "axum")]
pub use auth::{
    create_oauth_metadata, create_oauth_metadata_router, create_oauth_router,
    create_protected_resource_metadata, OAuthRouterOptions, OAuthRouterState,
};

#[cfg(feature = "websocket")]
pub use websocket::{WebSocketConfig, WebSocketError, WebSocketState, create_websocket_router, handle_websocket};
