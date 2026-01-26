use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;

pub async fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root_handler() -> &'static str {
    "Hello from Workspace API!"
}

async fn health_handler() -> &'static str {
    "OK"
}
