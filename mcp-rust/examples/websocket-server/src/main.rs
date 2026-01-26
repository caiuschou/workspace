//! Example: MCP WebSocket Server with Axum
//!
//! This example demonstrates how to run an MCP server over WebSocket,
//! providing full-duplex bidirectional communication.
//!
//! Features:
//! - Full-duplex WebSocket communication
//! - MCP subprotocol negotiation
//! - Automatic ping/pong handling
//!
//! Run with: cargo run -p mcp-websocket-server
//!
//! Test with websocat:
//! ```bash
//! websocat ws://localhost:8080/ws -H "Sec-WebSocket-Protocol: mcp"
//! ```
//!
//! Then send JSON-RPC messages:
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}
//! ```

use std::sync::Arc;

use mcp_core::protocol::RequestContext;
use mcp_core::types::{
    BaseMetadata, CallToolResult, ContentBlock, Icons, Implementation, ServerCapabilities,
    TextContent, Tool,
};
use mcp_server::{
    McpServer, ServerError, ServerOptions, WebSocketConfig, WebSocketState,
    create_websocket_router,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Create server info
    let server_info = Implementation {
        base: BaseMetadata {
            name: "mcp-websocket-server-example".to_string(),
            title: Some("MCP WebSocket Server Example".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some("Example MCP server with WebSocket transport".to_string()),
    };

    // Configure server capabilities
    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });
    server_options.instructions =
        Some("This is an example WebSocket MCP server with full-duplex communication.".to_string());

    // Create MCP server
    let mut mcp_server = McpServer::new(server_info, server_options);

    // Register example tools
    register_tools(&mut mcp_server)?;

    let mcp_server = Arc::new(mcp_server);

    // Configure WebSocket handler
    let config = WebSocketConfig {
        endpoint_path: "/ws".to_string(),
        enable_cors: true,
        channel_buffer_size: 100,
    };

    // Create handler state and router
    let state = Arc::new(WebSocketState::new(mcp_server, config));
    let app = create_websocket_router(state);

    // Start server
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("MCP WebSocket Server listening on ws://{}/ws", addr);
    println!();
    println!("Features:");
    println!("  - Full-duplex WebSocket communication");
    println!("  - MCP subprotocol negotiation");
    println!("  - Automatic ping/pong handling");
    println!();
    println!("Test with websocat:");
    println!(r#"  websocat ws://localhost:8080/ws -H "Sec-WebSocket-Protocol: mcp""#);
    println!();
    println!("Then send JSON-RPC messages:");
    println!(r#"  {{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-03-26","capabilities":{{}},"clientInfo":{{"name":"test","version":"0.1.0"}}}}}}"#);
    println!();

    axum::serve(listener, app).await?;

    Ok(())
}

fn register_tools(server: &mut McpServer) -> Result<(), Box<dyn std::error::Error>> {
    // Register an echo tool
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "echo".to_string(),
                title: Some("Echo Tool".to_string()),
            },
            icons: Icons::default(),
            description: Some("Echoes back the input message".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo"
                    }
                },
                "required": ["message"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        |params: Option<serde_json::Value>, _context: RequestContext| {
            Box::pin(async move {
                let message = params
                    .as_ref()
                    .and_then(|p| p.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no message)");

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Echo: {}",
                        message
                    )))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    // Register a greeting tool
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "greet".to_string(),
                title: Some("Greeting Tool".to_string()),
            },
            icons: Icons::default(),
            description: Some("Generates a greeting message".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    }
                },
                "required": ["name"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        |params: Option<serde_json::Value>, _context: RequestContext| {
            Box::pin(async move {
                let name = params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("World");

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Hello, {}! Welcome to MCP over WebSocket.",
                        name
                    )))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    // Register a time tool
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "current_time".to_string(),
                title: Some("Current Time".to_string()),
            },
            icons: Icons::default(),
            description: Some("Returns the current server time".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        |_params: Option<serde_json::Value>, _context: RequestContext| {
            Box::pin(async move {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Current Unix timestamp: {}",
                        now
                    )))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    Ok(())
}
