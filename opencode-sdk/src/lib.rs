//! OpenCode SDK for Rust
//!
//! Type-safe HTTP client for [OpenCode Server](https://opencodecn.com/docs/server) API.

pub mod client;
pub mod error;
pub mod event;
pub mod log;
pub mod open;
pub mod server;
pub mod session;

pub use client::{Client, HealthResponse};
pub use error::Error;
pub use log::init_logger;
pub use open::{OpenCode, OpenOptions, OpenResult, ServerHandle};
pub use session::{
    CreateSessionRequest, MessageInfo, MessageListItem, Part, SendMessageRequest, Session,
};
