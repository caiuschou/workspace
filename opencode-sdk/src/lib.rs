//! OpenCode SDK for Rust
//!
//! Type-safe HTTP client for [OpenCode Server](https://opencodecn.com/docs/server) API.

pub mod client;
pub mod error;

pub use client::{Client, HealthResponse};
pub use error::Error;
