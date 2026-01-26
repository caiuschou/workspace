use crate::{cli::ConfigCommands, config::ClientConfig, output::OutputFormatter, Result};
use crate::mcp_transport::McpServerClient;
use gitlab_mcp_server::Config as ServerConfig;

pub async fn execute_config(
    cmd: ConfigCommands,
    mcp_client: McpServerClient,
    formatter: OutputFormatter,
) -> Result<(McpServerClient, ())> {
    // Load client config
    let config = ClientConfig::load()?;

    match cmd {
        ConfigCommands::Show => {
            println!("\nGitLab MCP Configuration:");
            println!("========================");
            println!("GitLab URL: {}", config.gitlab_url);
            println!("Token: {}***", if config.gitlab_token.len() > 8 { &config.gitlab_token[..8] } else { &config.gitlab_token });
            println!("Output Format: {}", config.output_format);
            println!("Colors: {}", config.color);

            // Also show server config if exists
            if let Ok(path) = ServerConfig::config_file() {
                println!("\nServer Config File: {}", path.display());
                if path.exists() {
                    if let Ok(server_config) = ServerConfig::from_file(path) {
                        println!("Server GitLab URL: {}", server_config.gitlab_url);
                        println!("Server Token: {}***", if server_config.gitlab_token.len() > 8 {
                            &server_config.gitlab_token[..8]
                        } else {
                            &server_config.gitlab_token
                        });
                        println!("Server Log Level: {}", server_config.log_level);
                    }
                }
            }

            // Show MCP server configuration
            println!("\nMCP Server:");
            println!("-----------");
            println!("Command: {}", std::env::var("GITLAB_MCP_SERVER").unwrap_or_else(|_| "gitlab-mcp-server".to_string()));
            println!("Args: {}", std::env::var("GITLAB_MCP_SERVER_ARGS").unwrap_or_else(|_| "(none)".to_string()));
            println!();
        }

        ConfigCommands::SetUrl { url } => {
            let mut config = config;
            config.gitlab_url = url.clone();
            config.save()?;
            formatter.success(&format!("GitLab URL set to: {}", url));
        }

        ConfigCommands::SetToken { token } => {
            let mut config = config;
            config.gitlab_token = token.clone();
            config.save()?;
            formatter.success("Token saved to config file");

            // Also save to server config
            let mut server_config = ServerConfig::default();
            if let Ok(path) = ServerConfig::config_file() {
                if path.exists() {
                    if let Ok(existing) = ServerConfig::from_file(path.clone()) {
                        server_config = existing;
                    }
                }
            }
            server_config.gitlab_token = token;
            server_config.gitlab_url = config.gitlab_url.clone();
            server_config.save()?;
            formatter.success("Token also saved to server config file for AI assistant integration");
        }

        ConfigCommands::SetLogLevel { level } => {
            let mut server_config = ServerConfig::default();
            if let Ok(path) = ServerConfig::config_file() {
                if path.exists() {
                    if let Ok(existing) = ServerConfig::from_file(path.clone()) {
                        server_config = existing;
                    }
                }
            }
            server_config.log_level = level.clone();
            server_config.save()?;
            formatter.success(&format!("Log level set to: {}", level));
        }

        ConfigCommands::Path => {
            println!("\nConfig file locations:");
            println!("=====================");
            println!("Client config: {}", ClientConfig::config_file()?.display());
            println!("Server config: {}", ServerConfig::config_file()?.display());
            println!();

            // Show server config status
            if let Ok(path) = ServerConfig::config_file() {
                if path.exists() {
                    println!("Server config exists and will be used by the MCP server.");
                } else {
                    println!("Server config does not exist yet.");
                    println!("Run: gitlab-mcp config set-token <your-token> to create it.");
                }
            }
        }
    }

    Ok((mcp_client, ()))
}
