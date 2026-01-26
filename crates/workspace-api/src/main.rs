use workspace_api::run_server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "workspace_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = 3000;
    println!("Starting server on port {}", port);
    run_server(port).await?;

    Ok(())
}
