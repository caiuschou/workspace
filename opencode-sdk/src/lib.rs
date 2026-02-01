//! OpenCode SDK for Rust
//!
//! Type-safe HTTP client for [OpenCode Server](https://opencodecn.com/docs/server) API.

pub mod agent_skill;
pub mod api_log;
pub mod auth;
pub mod client;
mod request;
pub mod command;
pub mod config;
pub mod error;
pub mod event;
pub mod experimental;
pub mod file;
pub mod find;
pub mod instance;
pub mod lsp_mcp;
pub mod log;
pub mod open;
pub mod path_vcs;
pub mod permission;
pub mod project;
pub mod provider;
pub mod pty;
pub mod question;
pub mod server;
pub mod session;
pub mod tui;

pub use api_log::LogEntryRequest;
pub use client::{Client, HealthResponse};
pub use error::Error;
pub use event::{subscribe_and_stream, subscribe_and_stream_until_done, subscribe_global_events};
pub use log::init_logger;
pub use open::{OpenCode, OpenOptions, OpenResult, ServerHandle};
pub use permission::PermissionReplyRequest;
pub use project::{Project, UpdateProjectRequest};
pub use provider::{OAuthAuthorizeRequest, OAuthCallbackRequest};
pub use pty::{CreatePtyRequest, UpdatePtyRequest};
pub use session::{
    CreateSessionRequest, MessageInfo, MessageListItem, Part, SendMessageRequest, Session,
    SessionListParams,
};
