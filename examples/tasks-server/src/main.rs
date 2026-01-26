//! Example: MCP Tasks API Server
//!
//! This example demonstrates the MCP Tasks API for long-running operations.
//! When a tool is called with `task` metadata, it returns a task ID immediately
//! and the caller can poll for results.
//!
//! Features:
//! - Async tool execution with task tracking
//! - Task status polling (tasks/get)
//! - Task result retrieval (tasks/result)
//! - Task listing (tasks/list)
//! - Task cancellation (tasks/cancel)
//!
//! Run with: cargo run -p mcp-tasks-server
//!
//! Test with curl:
//! ```bash
//! # Initialize
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}'
//!
//! # Call a tool as a task (note the "task" field in params)
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"slow_operation","arguments":{"duration_secs":3},"task":{"ttl":60000}}}'
//!
//! # Get task status (replace TASK_ID with the actual task ID from the previous response)
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":3,"method":"tasks/get","params":{"taskId":"TASK_ID"}}'
//!
//! # Get task result
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":4,"method":"tasks/result","params":{"taskId":"TASK_ID"}}'
//!
//! # List all tasks
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":5,"method":"tasks/list","params":{}}'
//!
//! # Cancel a task
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":6,"method":"tasks/cancel","params":{"taskId":"TASK_ID"}}'
//! ```

use std::sync::Arc;
use std::time::Duration;

use mcp_core::protocol::{ProtocolOptions, RequestContext};
use mcp_core::types::{
    BaseMetadata, CallToolResult, ContentBlock, Icons, Implementation, ServerCapabilities,
    TextContent, Tool,
};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, InMemoryTaskStore, McpServer, ServerError, ServerOptions,
    create_router,
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
            name: "mcp-tasks-server".to_string(),
            title: Some("MCP Tasks API Server Example".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some("Example MCP server demonstrating Tasks API for async operations".to_string()),
    };

    // Create task store for managing async tasks
    let task_store = Arc::new(InMemoryTaskStore::default());

    // Configure server capabilities and options
    let server_options = ServerOptions {
        capabilities: Some(ServerCapabilities {
            tools: Some(mcp_core::types::ToolCapabilities {
                list_changed: Some(true),
            }),
            ..Default::default()
        }),
        instructions: Some(
            "This server demonstrates async task execution. Call tools with 'task' metadata to get a task ID for polling.".to_string()
        ),
        protocol_options: Some(ProtocolOptions {
            task_store: Some(task_store),
            ..Default::default()
        }),
    };

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

    println!("MCP Tasks API Server listening on http://{}", addr);
    println!();
    println!("This server demonstrates the Tasks API for async operations.");
    println!();
    println!("Available tools:");
    println!("  - slow_operation: Simulates a long-running operation");
    println!("  - compute_fibonacci: Computes Fibonacci numbers");
    println!("  - process_data: Simulates data processing with progress");
    println!();
    println!("To use the Tasks API, include a 'task' field in your tools/call params:");
    println!(r#"  {{"name": "slow_operation", "arguments": {{}}, "task": {{"ttl": 60000}}}}"#);
    println!();
    println!("Then use these methods to manage the task:");
    println!("  - tasks/get: Get task status");
    println!("  - tasks/result: Get task result (when completed)");
    println!("  - tasks/list: List all tasks");
    println!("  - tasks/cancel: Cancel a running task");
    println!();

    axum::serve(listener, app).await?;

    Ok(())
}

fn register_tools(server: &mut McpServer) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Slow Operation Tool - simulates a long-running task
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "slow_operation".to_string(),
                title: Some("Slow Operation".to_string()),
            },
            icons: Icons::default(),
            description: Some("Simulates a long-running operation. Use with task metadata for async execution.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "duration_secs": {
                        "type": "integer",
                        "description": "How long the operation should take (1-10 seconds)",
                        "minimum": 1,
                        "maximum": 10,
                        "default": 3
                    },
                    "message": {
                        "type": "string",
                        "description": "Optional message to include in the result"
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
                let duration_secs = params
                    .as_ref()
                    .and_then(|p| p.get("duration_secs"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3)
                    .min(10);

                let message = params
                    .as_ref()
                    .and_then(|p| p.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Operation completed");

                // Simulate long-running operation
                tokio::time::sleep(Duration::from_secs(duration_secs)).await;

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "{} (took {} seconds)",
                        message, duration_secs
                    )))],
                    structured_content: Some(json!({
                        "status": "completed",
                        "duration_secs": duration_secs,
                        "message": message
                    })),
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    // 2. Fibonacci Tool - compute-intensive operation
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "compute_fibonacci".to_string(),
                title: Some("Compute Fibonacci".to_string()),
            },
            icons: Icons::default(),
            description: Some("Computes the Nth Fibonacci number. Large values take longer.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "n": {
                        "type": "integer",
                        "description": "Which Fibonacci number to compute (1-40)",
                        "minimum": 1,
                        "maximum": 40
                    }
                },
                "required": ["n"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        |params: Option<serde_json::Value>, _context: RequestContext| {
            Box::pin(async move {
                let n = params
                    .as_ref()
                    .and_then(|p| p.get("n"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10)
                    .min(40) as u32;

                // Compute Fibonacci (intentionally slow for demonstration)
                let result = tokio::task::spawn_blocking(move || fibonacci(n))
                    .await
                    .map_err(|e| ServerError::Handler(e.to_string()))?;

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Fibonacci({}) = {}",
                        n, result
                    )))],
                    structured_content: Some(json!({
                        "n": n,
                        "result": result
                    })),
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    // 3. Process Data Tool - simulates batch processing
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "process_data".to_string(),
                title: Some("Process Data".to_string()),
            },
            icons: Icons::default(),
            description: Some("Simulates processing a batch of data items.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Data items to process"
                    },
                    "delay_ms": {
                        "type": "integer",
                        "description": "Delay between items in milliseconds",
                        "default": 500
                    }
                },
                "required": ["items"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        |params: Option<serde_json::Value>, _context: RequestContext| {
            Box::pin(async move {
                let items: Vec<String> = params
                    .as_ref()
                    .and_then(|p| p.get("items"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                let delay_ms = params
                    .as_ref()
                    .and_then(|p| p.get("delay_ms"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(500);

                let mut results = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    // Simulate processing
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    results.push(json!({
                        "index": i,
                        "item": item,
                        "processed": format!("Processed: {}", item.to_uppercase())
                    }));
                }

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Processed {} items successfully",
                        results.len()
                    )))],
                    structured_content: Some(json!({
                        "total_items": items.len(),
                        "results": results
                    })),
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    // 4. Quick Echo Tool - for comparison with sync execution
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "quick_echo".to_string(),
                title: Some("Quick Echo".to_string()),
            },
            icons: Icons::default(),
            description: Some("A fast tool for comparison. Returns immediately without task overhead.".to_string()),
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

/// Compute Fibonacci number (intentionally using recursive method for demo)
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let c = a.saturating_add(b);
                a = b;
                b = c;
            }
            b
        }
    }
}
