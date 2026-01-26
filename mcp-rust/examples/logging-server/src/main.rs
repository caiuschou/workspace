//! Example: MCP Logging Server
//!
//! This example demonstrates the MCP logging functionality:
//! - Sending log messages from server to client via notifications
//! - Handling logging/setLevel requests from clients
//!
//! Features:
//! - Server-initiated log messages at different levels
//! - Client can set minimum log level via logging/setLevel
//! - Tools that generate log messages during execution
//!
//! Run with: cargo run -p mcp-logging-server
//!
//! Test with curl:
//! ```bash
//! # Initialize (logging capability declared)
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}'
//!
//! # Set log level to 'debug'
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":2,"method":"logging/setLevel","params":{"level":"debug"}}'
//!
//! # Set log level to 'warning' (fewer messages)
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":3,"method":"logging/setLevel","params":{"level":"warning"}}'
//!
//! # Call a tool that generates log messages
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"process_with_logging","arguments":{"steps":3}}}'
//! ```

use std::sync::Arc;
use std::time::Duration;

use mcp_core::protocol::RequestContext;
use mcp_core::types::{
    BaseMetadata, CallToolResult, CapabilityFlag, ContentBlock, Icons, Implementation,
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
            name: "mcp-logging-server".to_string(),
            title: Some("MCP Logging Server Example".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some("Example MCP server demonstrating logging functionality".to_string()),
    };

    // Configure server capabilities with logging support
    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        logging: Some(CapabilityFlag {}), // Enable logging capability
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });
    server_options.instructions = Some(
        "This server demonstrates MCP logging. Use logging/setLevel to control log verbosity."
            .to_string(),
    );

    // Create MCP server
    let mut mcp_server = McpServer::new(server_info, server_options);

    // Register tools that demonstrate logging
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

    println!("MCP Logging Server listening on http://{}", addr);
    println!();
    println!("This server demonstrates MCP logging functionality.");
    println!();
    println!("Log Levels (from most to least verbose):");
    println!("  debug -> info -> notice -> warning -> error -> critical -> alert -> emergency");
    println!();
    println!("Available tools:");
    println!("  - process_with_logging: Demonstrates logging during tool execution");
    println!("  - log_test: Sends log messages at all levels");
    println!();
    println!("Example requests:");
    println!();
    println!("  # Set log level to debug (most verbose)");
    println!(
        r#"  curl -X POST http://localhost:8080/mcp -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","id":1,"method":"logging/setLevel","params":{{"level":"debug"}}}}'"#
    );
    println!();
    println!("  # Set log level to warning (less verbose)");
    println!(
        r#"  curl -X POST http://localhost:8080/mcp -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","id":2,"method":"logging/setLevel","params":{{"level":"warning"}}}}'"#
    );
    println!();
    println!("  # Call a tool that generates log messages");
    println!(
        r#"  curl -X POST http://localhost:8080/mcp -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"process_with_logging","arguments":{{"steps":3}}}}}}'"#
    );
    println!();

    axum::serve(listener, app).await?;

    Ok(())
}

fn register_tools(server: &mut McpServer) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Process with Logging Tool
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "process_with_logging".to_string(),
                title: Some("Process with Logging".to_string()),
            },
            icons: Icons::default(),
            description: Some(
                "Simulates a multi-step process that generates log messages at various levels"
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "steps": {
                        "type": "integer",
                        "description": "Number of processing steps (1-10)",
                        "minimum": 1,
                        "maximum": 10,
                        "default": 3
                    },
                    "fail_at_step": {
                        "type": "integer",
                        "description": "Optional: step number at which to simulate a warning/error"
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
                let steps = params
                    .as_ref()
                    .and_then(|p| p.get("steps"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3)
                    .min(10) as usize;

                let fail_at = params
                    .as_ref()
                    .and_then(|p| p.get("fail_at_step"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);

                // Note: In a real implementation, these would be sent as notifications
                // to the client. This example shows the structure of what would be logged.
                let mut log_messages = Vec::new();

                log_messages.push(format!("[DEBUG] Starting process with {} steps", steps));
                log_messages.push("[INFO] Process initialized successfully".to_string());

                for step in 1..=steps {
                    log_messages.push(format!("[DEBUG] Beginning step {}/{}", step, steps));

                    // Simulate processing
                    tokio::time::sleep(Duration::from_millis(200)).await;

                    if Some(step) == fail_at {
                        log_messages.push(format!(
                            "[WARNING] Step {} encountered recoverable issue, retrying...",
                            step
                        ));
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        log_messages.push(format!("[INFO] Step {} completed after retry", step));
                    } else {
                        log_messages.push(format!("[INFO] Step {} completed successfully", step));
                    }
                }

                log_messages.push("[NOTICE] All steps completed".to_string());
                log_messages.push(format!("[DEBUG] Process finished, total steps: {}", steps));

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Process completed successfully.\n\nLog output:\n{}",
                        log_messages.join("\n")
                    )))],
                    structured_content: Some(json!({
                        "status": "completed",
                        "total_steps": steps,
                        "logs": log_messages
                    })),
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    // 2. Log Test Tool - demonstrates all log levels
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "log_test".to_string(),
                title: Some("Log Test".to_string()),
            },
            icons: Icons::default(),
            description: Some("Generates test log messages at all severity levels".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "include_high_severity": {
                        "type": "boolean",
                        "description": "Whether to include critical/alert/emergency level logs",
                        "default": false
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
                let include_high = params
                    .as_ref()
                    .and_then(|p| p.get("include_high_severity"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut log_messages = vec![
                    ("debug", "This is a debug message - detailed diagnostic info"),
                    ("info", "This is an info message - general operational info"),
                    ("notice", "This is a notice - normal but significant event"),
                    ("warning", "This is a warning - something unexpected happened"),
                    ("error", "This is an error - something failed but we can continue"),
                ];

                if include_high {
                    log_messages.extend(vec![
                        ("critical", "This is critical - system component failed"),
                        ("alert", "This is an alert - immediate action required"),
                        ("emergency", "This is emergency - system is unusable"),
                    ]);
                }

                let formatted: Vec<String> = log_messages
                    .iter()
                    .map(|(level, msg)| format!("[{}] {}", level.to_uppercase(), msg))
                    .collect();

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Generated {} log messages:\n\n{}",
                        formatted.len(),
                        formatted.join("\n")
                    )))],
                    structured_content: Some(json!({
                        "log_count": formatted.len(),
                        "levels": log_messages.iter().map(|(l, _)| *l).collect::<Vec<_>>(),
                        "messages": log_messages.iter().map(|(l, m)| {
                            json!({"level": l, "message": m})
                        }).collect::<Vec<_>>()
                    })),
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    // 3. Simple echo tool for basic testing
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "echo".to_string(),
                title: Some("Echo".to_string()),
            },
            icons: Icons::default(),
            description: Some("Simple echo tool for testing".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
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

    Ok(())
}
