//! Example: Backwards-Compatible MCP Server (Streamable HTTP + Legacy SSE)
//!
//! This example demonstrates how to run an MCP server that supports both:
//! - Modern Streamable HTTP transport (2025-03-26)
//! - Legacy HTTP+SSE transport (2024-11-05)
//!
//! This allows the server to accept connections from both old and new MCP clients.
//!
//! ## Endpoints
//!
//! Modern Streamable HTTP (2025-03-26):
//! - POST /mcp - Send JSON-RPC messages
//! - GET /mcp - Establish SSE connection
//! - DELETE /mcp - Close session
//!
//! Legacy SSE (2024-11-05):
//! - GET /sse - Establish SSE connection (receives `endpoint` event)
//! - POST /message?sessionId=xxx - Send JSON-RPC messages
//!
//! Run with: cargo run -p mcp-sse-compat-server

use std::sync::Arc;

use mcp_core::protocol::RequestContext;
use mcp_core::types::{
    BaseMetadata, CallToolResult, ContentBlock, Icons, Implementation, ServerCapabilities,
    TextContent, Tool,
};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, LegacySseConfig, LegacySseState, McpServer, ServerError,
    ServerOptions, create_legacy_sse_router, create_router,
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
            name: "mcp-sse-compat-server".to_string(),
            title: Some("MCP Backwards-Compatible Server".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some(
            "Example MCP server supporting both Streamable HTTP and legacy SSE".to_string(),
        ),
    };

    // Configure server capabilities
    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });
    server_options.instructions = Some(
        "This server supports both modern Streamable HTTP and legacy SSE transports.".to_string(),
    );

    // Create MCP server
    let mut mcp_server = McpServer::new(server_info, server_options);

    // Register example tools
    register_tools(&mut mcp_server)?;

    let mcp_server = Arc::new(mcp_server);

    // Create modern Streamable HTTP router
    let streamable_config = AxumHandlerConfig {
        endpoint_path: "/mcp".to_string(),
        enable_cors: true,
        ..Default::default()
    };
    let streamable_state = Arc::new(AxumHandlerState::new(Arc::clone(&mcp_server), streamable_config));
    let streamable_router = create_router(streamable_state);

    // Create legacy SSE router
    let legacy_config = LegacySseConfig {
        endpoint_path: "/sse".to_string(),
        message_path: "/message".to_string(),
    };
    let legacy_state = Arc::new(LegacySseState::new(Arc::clone(&mcp_server), legacy_config));
    let legacy_router = create_legacy_sse_router(legacy_state);

    // Merge routers
    let app = streamable_router.merge(legacy_router);

    // Start server
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("MCP Backwards-Compatible Server listening on http://{}", addr);
    println!();
    println!("This server supports both modern and legacy MCP clients:");
    println!();
    println!("Modern Streamable HTTP (2025-03-26):");
    println!("  POST   http://{}/mcp - Send JSON-RPC messages", addr);
    println!("  GET    http://{}/mcp - Establish SSE connection", addr);
    println!("  DELETE http://{}/mcp - Close session", addr);
    println!();
    println!("Legacy SSE (2024-11-05):");
    println!("  GET    http://{}/sse - Establish SSE connection", addr);
    println!("  POST   http://{}/message?sessionId=xxx - Send messages", addr);
    println!();
    println!("Test modern client:");
    println!(r#"  curl -X POST http://localhost:8080/mcp -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-03-26","capabilities":{{}},"clientInfo":{{"name":"test","version":"0.1.0"}}}}}}'"#);
    println!();
    println!("Test legacy client:");
    println!(r#"  curl -N http://localhost:8080/sse -H "Accept: text/event-stream""#);
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
                        "Hello, {}! This server supports both modern and legacy clients.",
                        name
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
