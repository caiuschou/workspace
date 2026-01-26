//! Streamline stdio helpers for the client binary.
pub mod auth;
pub mod client;
pub mod http;
pub mod stdio;
pub mod websocket;

pub use stdio::{
    DEFAULT_INHERITED_ENV_VARS, JsonRpcMessage, ReadBuffer, StdioClientTransport,
    StdioClientTransportError, StdioServerParameters, StdioStream, Transport, deserialize_message,
    get_default_environment, serialize_message,
};

pub use client::{Client, ClientCapabilities, ClientError, ClientOptions};

pub use http::{
    HttpClientConfig, HttpClientError, HttpClientTransport, LegacySseClientConfig,
    LegacySseClientTransport, ReconnectOptions,
};

#[cfg(feature = "websocket")]
pub use websocket::{WebSocketClientError, WebSocketClientTransport};

pub use auth::{
    auth, discover_authorization_server_metadata, discover_protected_resource_metadata,
    get_protected_resource_metadata_url, register_client, start_authorization, AuthOptions,
    AuthResult, InMemoryOAuthClientProvider, InvalidationScope, OAuthClientError,
    OAuthClientProvider,
};
