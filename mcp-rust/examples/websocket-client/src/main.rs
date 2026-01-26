//! Example: MCP WebSocket Client
//!
//! This example demonstrates how to connect to an MCP server over WebSocket.
//! It uses `WebSocketClientTransport` for full-duplex communication.
//!
//! Features:
//! - Full-duplex WebSocket communication
//! - MCP subprotocol negotiation
//! - Event-driven message handling
//!
//! Usage:
//!   1. First, start the WebSocket server: cargo run -p mcp-websocket-server
//!   2. Then run this client: cargo run -p mcp-websocket-client
//!
//! Or test with websocat:
//!   websocat ws://localhost:8080/ws -H "Sec-WebSocket-Protocol: mcp"

use std::sync::Arc;
use std::time::Duration;

use mcp_client::websocket::WebSocketClientTransport;
use mcp_core::stdio::JsonRpcMessage;
use mcp_core::types::{MessageId, NotificationMessage, RequestMessage};
use serde_json::json;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Client error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("MCP WebSocket Client Example");
    println!("=============================");
    println!();

    let url = "ws://localhost:8080/ws";
    println!("Connecting to: {}", url);
    println!();

    // Create the WebSocket transport
    let mut transport = WebSocketClientTransport::new(url);

    // Track received messages
    let messages: Arc<Mutex<Vec<JsonRpcMessage>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_for_handler = Arc::clone(&messages);

    // Set up event handlers
    transport.on_message(move |msg| {
        println!("[Received] {:?}", msg);
        let messages = Arc::clone(&messages_for_handler);
        tokio::spawn(async move {
            messages.lock().await.push(msg);
        });
    });

    transport.on_error(|err| {
        eprintln!("[Error] {:?}", err);
    });

    transport.on_close(|| {
        println!("[Connection closed]");
    });

    // Start the transport (establishes WebSocket connection)
    println!("Starting WebSocket transport...");
    transport.start().await?;
    println!("Connected!");
    println!();

    // Wait a bit for connection to stabilize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send initialize request
    println!("=== Sending initialize request ===");
    let init_request = RequestMessage::new(
        MessageId::Number(1),
        "initialize",
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "roots": {
                    "listChanged": true
                }
            },
            "clientInfo": {
                "name": "mcp-websocket-client-example",
                "version": "0.1.0"
            }
        }),
    );
    transport.send(&JsonRpcMessage::Request(init_request)).await?;
    println!("Sent initialize request");

    // Wait for response
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Send initialized notification
    println!();
    println!("=== Sending initialized notification ===");
    let initialized_notification = NotificationMessage::new(
        "notifications/initialized",
        Some(json!({})),
    );
    transport.send(&JsonRpcMessage::Notification(initialized_notification)).await?;
    println!("Sent initialized notification");

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(300)).await;

    // List available tools
    println!();
    println!("=== Listing tools ===");
    let list_tools_request = RequestMessage::new(
        MessageId::Number(2),
        "tools/list",
        json!({}),
    );
    transport.send(&JsonRpcMessage::Request(list_tools_request)).await?;
    println!("Sent tools/list request");

    // Wait for response
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Call the echo tool
    println!();
    println!("=== Calling echo tool ===");
    let echo_request = RequestMessage::new(
        MessageId::Number(3),
        "tools/call",
        json!({
            "name": "echo",
            "arguments": {
                "message": "Hello from WebSocket client!"
            }
        }),
    );
    transport.send(&JsonRpcMessage::Request(echo_request)).await?;
    println!("Sent tools/call request for 'echo'");

    // Wait for response
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Call the greet tool
    println!();
    println!("=== Calling greet tool ===");
    let greet_request = RequestMessage::new(
        MessageId::Number(4),
        "tools/call",
        json!({
            "name": "greet",
            "arguments": {
                "name": "WebSocket User"
            }
        }),
    );
    transport.send(&JsonRpcMessage::Request(greet_request)).await?;
    println!("Sent tools/call request for 'greet'");

    // Wait for response
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Call the current_time tool
    println!();
    println!("=== Calling current_time tool ===");
    let time_request = RequestMessage::new(
        MessageId::Number(5),
        "tools/call",
        json!({
            "name": "current_time",
            "arguments": {}
        }),
    );
    transport.send(&JsonRpcMessage::Request(time_request)).await?;
    println!("Sent tools/call request for 'current_time'");

    // Wait for all responses
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Print summary
    println!();
    println!("=== Summary ===");
    let received = messages.lock().await;
    println!("Total messages received: {}", received.len());

    for (i, msg) in received.iter().enumerate() {
        match msg {
            JsonRpcMessage::Result(result) => {
                println!(
                    "  [{}] Response id={:?}: {}",
                    i + 1,
                    result.id,
                    if result.error.is_some() { "ERROR" } else { "OK" }
                );
            }
            JsonRpcMessage::Notification(notif) => {
                println!("  [{}] Notification: {}", i + 1, notif.method);
            }
            JsonRpcMessage::Request(req) => {
                println!("  [{}] Request: {}", i + 1, req.method);
            }
        }
    }

    // Close the transport
    println!();
    println!("Closing WebSocket connection...");
    transport.close().await?;
    println!("Done!");

    Ok(())
}
