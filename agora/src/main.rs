//! Agora - Independent WebSocket Server
//!
//! A simple WebSocket server built with axum.
//! Supports echo and bidirectional communication.
//!
//! Run: cargo run
//! Test with websocat: websocat ws://127.0.0.1:8080/ws

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Binds to 127.0.0.1:8080 and starts the WebSocket server.
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "agora=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/ws", get(handle_ws_upgrade));

    let addr: SocketAddr = "127.0.0.1:8080".parse().expect("invalid address");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    tracing::info!("Agora WebSocket server listening on ws://{}/ws", addr);

    axum::serve(listener, app).await.expect("server error");
}

/// Handles WebSocket upgrade requests.
async fn handle_ws_upgrade(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

/// Handles an established WebSocket connection (echo server).
async fn handle_socket(mut socket: WebSocket) {
    tracing::info!("WebSocket client connected");

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("Received: {}", text);
                if socket
                    .send(Message::Text(format!("Echo: {}", text)))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("Client sent close");
                break;
            }
            Ok(Message::Ping(data)) => {
                if socket.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("WebSocket error: {:?}", e);
                break;
            }
        }
    }

    tracing::info!("WebSocket client disconnected");
}
