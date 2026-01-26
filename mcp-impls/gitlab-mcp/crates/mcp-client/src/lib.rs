//! GitLab MCP Client (CLI)
//!
//! Command-line tool for GitLab MCP operations.

pub mod cli;
pub mod config;
pub mod commands;
pub mod output;
pub mod mcp_transport;

// Re-export commonly used types
pub use cli::{Cli, Commands, ConfigCommands};
pub use config::ClientConfig;
pub use output::OutputFormatter;
pub use anyhow::Result;
