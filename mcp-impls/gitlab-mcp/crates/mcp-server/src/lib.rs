//! GitLab MCP Server
//!
//! MCP server for GitLab operations.

pub mod config;
pub mod error;
pub mod gitlab;
pub mod logging;
pub mod server;
pub mod tools;

pub use config::Config;
pub use error::{GitLabError, Result};
pub use gitlab::GitLabClient;
pub use server::GitLabMcpServer;
