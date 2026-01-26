//! Example: MCP HTTP Client with Streamable HTTP Transport
//!
//! This example demonstrates how to connect to an MCP server over HTTP.
//! It uses `HttpClientTransport` for communication.
//!
//! Usage:
//!   1. First, start the HTTP server: cargo run -p mcp-http-server
//!   2. Then run this client: cargo run -p mcp-http-client
//!
//! Or test with curl:
//!   curl -X POST http://localhost:8080/mcp \
//!        -H "Content-Type: application/json" \
//!        -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use mcp_client::http::{HttpClientConfig, HttpClientTransport, ReconnectOptions};
use mcp_core::stdio::JsonRpcMessage;
use mcp_core::types::{MessageId, RequestMessage};
use serde_json::json;

fn main() {
    if let Err(e) = run() {
        eprintln!("Client error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("MCP HTTP Client Example");
    println!("========================");
    println!();

    // Configure the HTTP client
    let config = HttpClientConfig::new("http://localhost:8080")
        .endpoint_path("/mcp")
        .request_timeout(Duration::from_secs(30))
        .reconnect_options(ReconnectOptions {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_attempts: Some(5),
            jitter: 0.1,
        })
        .auto_reconnect(true);

    println!("Connecting to: {}", config.endpoint_url());
    println!();

    // Create the transport
    let mut transport = HttpClientTransport::new(config);

    // Track received messages
    let messages: Arc<Mutex<Vec<JsonRpcMessage>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = Arc::clone(&messages);

    // Set up event handlers
    transport.on_message(move |msg| {
        println!("[Received] {:?}", msg);
        messages_clone.lock().unwrap().push(msg);
    });

    transport.on_error(|err| {
        eprintln!("[Error] {}", err);
    });

    transport.on_close(|| {
        println!("[Connection closed]");
    });

    // Start the transport (establishes SSE connection)
    println!("Starting transport...");
    transport.start()?;

    // Wait a bit for connection
    thread::sleep(Duration::from_millis(500));

    if let Some(session_id) = transport.session_id() {
        println!("Session established: {}", session_id);
    }
    println!();

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
                "name": "mcp-http-client-example",
                "version": "0.1.0"
            }
        }),
    );
    transport.send(&JsonRpcMessage::Request(init_request))?;
    println!("Sent initialize request");

    // Wait for response
    thread::sleep(Duration::from_millis(500));

    // Send initialized notification
    println!();
    println!("=== Sending initialized notification ===");
    let initialized_notification = mcp_core::types::NotificationMessage {
        jsonrpc: "2.0".to_string(),
        method: "notifications/initialized".to_string(),
        params: None,
    };
    transport.send(&JsonRpcMessage::Notification(initialized_notification))?;
    println!("Sent initialized notification");

    // Wait a bit
    thread::sleep(Duration::from_millis(300));

    // List available tools
    println!();
    println!("=== Listing tools ===");
    let list_tools_request = RequestMessage::new(
        MessageId::Number(2),
        "tools/list",
        json!({}),
    );
    transport.send(&JsonRpcMessage::Request(list_tools_request))?;
    println!("Sent tools/list request");

    // Wait for response
    thread::sleep(Duration::from_millis(500));

    // Call the echo tool
    println!();
    println!("=== Calling echo tool ===");
    let echo_request = RequestMessage::new(
        MessageId::Number(3),
        "tools/call",
        json!({
            "name": "echo",
            "arguments": {
                "message": "Hello from HTTP client!"
            }
        }),
    );
    transport.send(&JsonRpcMessage::Request(echo_request))?;
    println!("Sent tools/call request for 'echo'");

    // Wait for response
    thread::sleep(Duration::from_millis(500));

    // Call the greet tool
    println!();
    println!("=== Calling greet tool ===");
    let greet_request = RequestMessage::new(
        MessageId::Number(4),
        "tools/call",
        json!({
            "name": "greet",
            "arguments": {
                "name": "MCP User"
            }
        }),
    );
    transport.send(&JsonRpcMessage::Request(greet_request))?;
    println!("Sent tools/call request for 'greet'");

    // Wait for response
    thread::sleep(Duration::from_millis(500));

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
    transport.send(&JsonRpcMessage::Request(time_request))?;
    println!("Sent tools/call request for 'current_time'");

    // Wait for all responses
    thread::sleep(Duration::from_secs(1));

    // Print summary
    println!();
    println!("=== Summary ===");
    let received = messages.lock().unwrap();
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
    println!("Closing connection...");
    transport.close()?;
    println!("Done!");

    Ok(())
}
