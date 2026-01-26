//! HTTP/SSE integration tests.
//!
//! These tests verify the complete HTTP/SSE communication flow including:
//! - POST request/response
//! - SSE connection establishment
//! - Message broadcasting
//! - Last-Event-ID replay

#![cfg(feature = "axum")]

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use mcp_core::types::{BaseMetadata, Icons, Implementation, ServerCapabilities};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, McpServer, ServerOptions, create_router,
};
use tower::util::ServiceExt;

fn create_test_server() -> Arc<McpServer> {
    let server_info = Implementation {
        base: BaseMetadata {
            name: "test-server".to_string(),
            title: None,
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: None,
    };

    let mut options = ServerOptions::default();
    options.capabilities = Some(ServerCapabilities {
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });

    Arc::new(McpServer::new(server_info, options))
}

fn create_test_state() -> Arc<AxumHandlerState> {
    let server = create_test_server();
    let config = AxumHandlerConfig {
        base_url: Some("http://localhost:8080".to_string()),
        endpoint_path: "/mcp".to_string(),
        keep_alive_interval: Duration::from_secs(30),
        broadcast_capacity: 100,
        enable_cors: true,
        ..Default::default()
    };
    Arc::new(AxumHandlerState::new(server, config))
}

#[tokio::test]
async fn test_post_initialize() {
    let state = create_test_state();
    let app = create_router(state);

    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(initialize_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Should have session ID in response header
    assert!(response.headers().contains_key("mcp-session-id"));

    // Parse response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["result"].is_object());
}

#[tokio::test]
async fn test_post_invalid_content_type() {
    let state = create_test_state();
    let app = create_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_post_invalid_json() {
    let state = create_test_state();
    let app = create_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("not valid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_sse_connection() {
    let state = create_test_state();
    let app = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/mcp")
        .header(header::ACCEPT, "text/event-stream")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Should have SSE content type
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("text/event-stream"));

    // Should have session ID
    assert!(response.headers().contains_key("mcp-session-id"));
}

#[tokio::test]
async fn test_get_sse_wrong_accept() {
    let state = create_test_state();
    let app = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/mcp")
        .header(header::ACCEPT, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[tokio::test]
async fn test_delete_session() {
    let state = create_test_state();
    let app = create_router(state.clone());

    // First create a session via POST
    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(initialize_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let session_id = response
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Now delete the session
    let app = create_router(state);
    let request = Request::builder()
        .method("DELETE")
        .uri("/mcp")
        .header("mcp-session-id", &session_id)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_missing_session_id() {
    let state = create_test_state();
    let app = create_router(state);

    let request = Request::builder()
        .method("DELETE")
        .uri("/mcp")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_session_persistence() {
    let state = create_test_state();

    // Create session
    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        }
    });

    let app = create_router(state.clone());
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(initialize_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let session_id = response
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Send another request with the same session ID
    let tools_list_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let app = create_router(state);
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::CONTENT_TYPE, "application/json")
        .header("mcp-session-id", &session_id)
        .body(Body::from(tools_list_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Should NOT have session ID in response (existing session)
    assert!(!response.headers().contains_key("mcp-session-id"));
}

#[tokio::test]
async fn test_event_buffer() {
    use mcp_server::{EventBuffer, EventBufferConfig, BufferedEvent};
    use mcp_core::http::SseEvent;

    let config = EventBufferConfig {
        max_events: 5,
        max_age_secs: 300,
    };
    let mut buffer = EventBuffer::new(config);

    // Add events
    for i in 0..7 {
        buffer.push(BufferedEvent::new(
            format!("event-{}", i),
            SseEvent::Ping,
        ));
    }

    // Should only have 5 events (max capacity)
    assert_eq!(buffer.len(), 5);

    // Should have events 2-6 (oldest removed)
    let events = buffer.all_events();
    assert_eq!(events[0].id, "event-2");
    assert_eq!(events[4].id, "event-6");

    // Test events_after
    let after = buffer.events_after("event-3");
    assert_eq!(after.len(), 3);
    assert_eq!(after[0].id, "event-4");
}

#[tokio::test]
async fn test_broadcaster() {
    use mcp_server::SseBroadcaster;
    use mcp_core::stdio::JsonRpcMessage;
    use mcp_core::types::ResultMessage;

    let broadcaster = SseBroadcaster::new("test-session".to_string(), 10);

    // Subscribe
    let mut rx = broadcaster.subscribe();

    // Send a message
    let message = JsonRpcMessage::Result(ResultMessage::success(
        mcp_core::types::MessageId::Number(1),
        serde_json::json!({"test": true}),
    ));

    let event_id = broadcaster.send_message(message).unwrap();
    assert!(event_id.starts_with("test-session-"));

    // Receive the message
    let received = rx.recv().await.unwrap();
    match received {
        mcp_core::http::SseEvent::Message { id, .. } => {
            assert_eq!(id, Some(event_id.clone()));
        }
        _ => panic!("Expected Message event"),
    }

    // Check buffered events
    let buffered = broadcaster.get_all_buffered_events();
    assert_eq!(buffered.len(), 1);
    assert_eq!(buffered[0].id, event_id);

    // Test replay
    let replay = broadcaster.get_replay_events("nonexistent");
    assert_eq!(replay.len(), 1);
}

#[tokio::test]
async fn test_cors_headers() {
    let state = create_test_state();
    let app = create_router(state);

    // Send OPTIONS request (CORS preflight)
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/mcp")
        .header("Origin", "http://example.com")
        .header("Access-Control-Request-Method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should have CORS headers
    assert!(response.headers().contains_key("access-control-allow-origin"));
    assert!(response.headers().contains_key("access-control-allow-methods"));
}
