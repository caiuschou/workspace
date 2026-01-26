//! HTTP transport module for MCP.
//!
//! This module provides types and traits for HTTP-based transports using
//! Server-Sent Events (SSE) for bidirectional communication.

mod error;
mod session;
mod sse;
mod transport;

pub use error::HttpTransportError;
pub use session::{ResumptionToken, ResumptionTokenError, SessionId};
pub use sse::{
    headers, ParsedSseEvent, SseEvent, SseEventParseError, SseHeaders, SseParser,
};
pub use transport::{AsyncTransport, ConnectionState, MessageReceiver};
