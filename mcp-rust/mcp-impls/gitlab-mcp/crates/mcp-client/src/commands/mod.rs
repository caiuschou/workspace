pub mod project;
pub mod config;

use crate::{Cli, OutputFormatter, Result, Commands};
use crate::mcp_transport::McpServerClient;
use clap::Parser;

pub use project::*;
pub use config::*;

/// Execute a command
pub async fn execute() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    }

    let formatter = OutputFormatter::new(&cli.output, cli.color);

    // Start MCP client connection to server
    // Default to using "gitlab-mcp-server" in PATH
    let server_command = std::env::var("GITLAB_MCP_SERVER")
        .unwrap_or_else(|_| "gitlab-mcp-server".to_string());
    let server_args: Vec<String> = std::env::var("GITLAB_MCP_SERVER_ARGS")
        .unwrap_or_default()
        .split_whitespace()
        .map(String::from)
        .collect();

    let mcp_client = McpServerClient::start(&server_command, &server_args)?;

    // Execute command and get back the client for cleanup
    let mcp_client = match cli.command {
        Commands::Config(cmd) => {
            let (client, _) = execute_config(cmd, mcp_client, formatter).await?;
            client
        }
        Commands::Project(cmd) => {
            let (client, _) = execute_project(cmd, mcp_client, formatter).await?;
            client
        }
        _ => {
            mcp_client.close()?;
            formatter.error("Command not implemented yet");
            std::process::exit(1);
        }
    };

    // Close the MCP connection
    mcp_client.close()?;

    Ok(())
}
