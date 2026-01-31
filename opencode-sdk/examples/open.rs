//! Example: OpenCode::open - connect to or start the server.
//!
//! Run with: cargo run --example open
//!
//! This example uses auto_start(false) to connect to an existing server only.
//! If no server is running, it will fail. To auto-start the server with
//! project path and chat content, use:
//!
//!   OpenCode::open(
//!       OpenOptions::default()
//!           .project_path(".")
//!           .chat_content("分析项目结构")
//!   )

use opencode_sdk::{OpenCode, OpenOptions};

#[tokio::main]
async fn main() -> Result<(), opencode_sdk::Error> {
    // Connect to existing server only (no auto-start)
    let result = OpenCode::open(OpenOptions::default().auto_start(false)).await?;

    let health = result.client.health().await?;
    println!("OpenCode server version: {}", health.version);
    if let Some(s) = result.server {
        s.shutdown();
    }
    Ok(())
}
