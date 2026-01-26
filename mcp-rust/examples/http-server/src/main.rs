//! Example: MCP HTTP Server with Axum and SSE Streaming
//!
//! This example demonstrates how to run an MCP server over HTTP with true SSE streaming,
//! bidirectional communication, and Last-Event-ID replay support.
//!
//! Features:
//! - True SSE streaming (long-lived connections)
//! - Server-initiated push via broadcast channels
//! - Last-Event-ID replay for reconnection
//! - CORS support
//!
//! Endpoints:
//! - POST /mcp - Send JSON-RPC messages
//! - GET /mcp - Establish SSE connection for receiving messages
//! - DELETE /mcp - Close session
//!
//! Run with: cargo run -p mcp-http-server
//!
//! Test with curl:
//! ```bash
//! # Initialize
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}'
//!
//! # Establish SSE connection
//! curl -N http://localhost:8080/mcp -H "Accept: text/event-stream"
//!
//! # List tools
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
//!
//! # Call a tool
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello!"}}}'
//! ```

use std::sync::Arc;
use std::time::Duration;

use mcp_core::protocol::RequestContext;
use mcp_core::types::{
    BaseMetadata, CallToolResult, ContentBlock, Icons, Implementation, ResourceLink,
    ServerCapabilities, TextContent, Tool,
};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, McpServer, ServerError, ServerOptions, create_router,
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
            name: "mcp-http-server-example".to_string(),
            title: Some("MCP HTTP Server Example".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some("Example MCP server with HTTP/SSE transport".to_string()),
    };

    // Configure server capabilities
    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });
    server_options.instructions = Some("This is an example HTTP MCP server with SSE streaming.".to_string());

    // Create MCP server
    let mut mcp_server = McpServer::new(server_info, server_options);

    // Register example tools
    register_tools(&mut mcp_server)?;

    let mcp_server = Arc::new(mcp_server);

    // Configure HTTP handler
    let config = AxumHandlerConfig {
        base_url: Some("http://localhost:8080".to_string()),
        endpoint_path: "/mcp".to_string(),
        keep_alive_interval: Duration::from_secs(30),
        broadcast_capacity: 100,
        enable_cors: true,
        ..Default::default()
    };

    // Create handler state and router
    let state = Arc::new(AxumHandlerState::new(mcp_server, config));
    let app = create_router(state);

    // Start server
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("MCP HTTP Server listening on http://{}", addr);
    println!();
    println!("Features:");
    println!("  - True SSE streaming (long-lived connections)");
    println!("  - Server-initiated push via broadcast channels");
    println!("  - Last-Event-ID replay for reconnection");
    println!("  - CORS support");
    println!();
    println!("Endpoints:");
    println!("  POST   http://{}/mcp - Send JSON-RPC messages", addr);
    println!("  GET    http://{}/mcp - Establish SSE connection", addr);
    println!("  DELETE http://{}/mcp - Close session", addr);
    println!();
    println!("Example requests:");
    println!();
    println!("  # Initialize");
    println!(r#"  curl -X POST http://localhost:8080/mcp \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-03-26","capabilities":{{}},"clientInfo":{{"name":"test","version":"0.1.0"}}}}}}'"#);
    println!();
    println!("  # Establish SSE connection");
    println!(r#"  curl -N http://localhost:8080/mcp -H "Accept: text/event-stream""#);
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
                        "Hello, {}! Welcome to MCP over HTTP with SSE streaming.",
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

    // Register a tool that returns ResourceLinks
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "list_files".to_string(),
                title: Some("List Files".to_string()),
            },
            icons: Icons::default(),
            description: Some("Returns a list of files as ResourceLinks without embedding their content".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "include_descriptions": {
                        "type": "boolean",
                        "description": "Whether to include descriptions in the resource links"
                    }
                }
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        |params: Option<serde_json::Value>, _context: RequestContext| {
            Box::pin(async move {
                let include_descriptions = params
                    .as_ref()
                    .and_then(|p| p.get("include_descriptions"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                // Create ResourceLinks for example files
                let mut link1 = ResourceLink::with_uri("file:///example/file1.txt", "example-file-1")
                    .mime_type("text/plain");
                let mut link2 = ResourceLink::with_uri("file:///example/file2.txt", "example-file-2")
                    .mime_type("text/plain");
                let mut link3 = ResourceLink::with_uri("https://example.com/data.json", "remote-data")
                    .mime_type("application/json");

                if include_descriptions {
                    link1 = link1.description("First example file");
                    link2 = link2.description("Second example file");
                    link3 = link3.description("Remote JSON data");
                }

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![
                        ContentBlock::Text(TextContent::new(
                            "Here are the available files as resource links:"
                        )),
                        ContentBlock::ResourceLink(link1),
                        ContentBlock::ResourceLink(link2),
                        ContentBlock::ResourceLink(link3),
                    ],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    Ok(())
}
