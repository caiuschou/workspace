use std::io::{self, BufRead, BufReader, Write};
use gitlab_mcp_server::{GitLabMcpServer, logging};
use mcp_core::stdio::{JsonRpcMessage, serialize_message};
use mcp_core::types::{Implementation, BaseMetadata, Icons, ServerCapabilities};

fn main() -> anyhow::Result<()> {
    // Create Tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;
    // Load .env file if present
    dotenv::dotenv().ok();

    // Initialize logging with file output to ~/.mcp/logs/
    // The guard must be kept alive for the program duration
    let _log_guard = logging::init_logging()?;

    tracing::info!("GitLab MCP Server starting (version {})", env!("CARGO_PKG_VERSION"));

    // Create server info
    let server_info = Implementation {
        base: BaseMetadata {
            name: "gitlab-mcp-server".to_string(),
            title: Some("GitLab MCP Server".to_string()),
        },
        icons: Icons::default(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        website_url: Some("https://gitlab.com".to_string()),
        description: Some("MCP server for GitLab operations".to_string()),
    };

    // Create server options
    let mut server_options = mcp_server::ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });
    server_options.instructions = Some(
        "GitLab MCP server providing tools for managing GitLab projects, issues, merge requests, pipelines, and more.".to_string()
    );

    // Create MCP server
    let mut server = mcp_server::McpServer::new(server_info, server_options);

    // Register tools
    match GitLabMcpServer::register_tools(&mut server) {
        Ok(_) => {
            tracing::info!("Tools registered successfully");
        }
        Err(e) => {
            tracing::error!("Failed to register some tools: {}", e);
        }
    }

    // Stdio loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();
    let mut read_buffer = mcp_core::stdio::ReadBuffer::default();

    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        read_buffer.append(buffer.as_bytes());
        while let Ok(Some(message)) = read_buffer.read_message() {
            match message {
                JsonRpcMessage::Request(request) => {
                    match rt.block_on(server.server().handle_request(request, None)) {
                        Ok(response) => {
                            let response_msg = JsonRpcMessage::Result(response);
                            match serialize_message(&response_msg) {
                                Ok(serialized) => {
                                    if let Err(e) = stdout.write_all(serialized.as_bytes()) {
                                        eprintln!("[gitlab-mcp-server] Error writing response: {}", e);
                                        break;
                                    }
                                    if let Err(e) = stdout.flush() {
                                        eprintln!("[gitlab-mcp-server] Error flushing: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Error serializing response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error handling request: {}", e);
                        }
                    }
                }
                JsonRpcMessage::Notification(notification) => {
                    if let Err(e) = rt.block_on(server.server().handle_notification(notification, None)) {
                        tracing::error!("Error handling notification: {}", e);
                    }
                }
                JsonRpcMessage::Result(result) => {
                    tracing::debug!("Received result: {:?}", result);
                }
            }
        }
    }

    tracing::info!("Server shutdown");
    Ok(())
}
