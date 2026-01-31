//! CLI example: open OpenCode in /tmp/example1 and request a Python Fibonacci script.
//!
//! Run with: cargo run --example cli_fibonacci
//!
//! Prerequisites:
//! - `opencode` in PATH, or set OPENCODE_CMD; if not found, auto-installs via npm
//! - /tmp/example1 will be used as project directory (created if needed)

use opencode_sdk::{init_logger, OpenCode, OpenOptions};

const PROJECT_DIR: &str = "/tmp/example1";
const CHAT_CONTENT: &str = "写一个 python 的斐波那契数列的代码";

fn opencode_command() -> String {
    std::env::var("OPENCODE_CMD").unwrap_or_else(|_| "opencode".to_string())
}

#[tokio::main]
async fn main() -> Result<(), opencode_sdk::Error> {
    let _guard = init_logger(None); // Hold guard to keep file logging active

    // Ensure project directory exists
    std::fs::create_dir_all(PROJECT_DIR).expect("create project dir");

    let cmd = opencode_command();
    println!("Opening OpenCode...");
    println!("  command: {}", cmd);
    println!("  project: {}", PROJECT_DIR);
    println!("  prompt:  {}", CHAT_CONTENT);

    let result = OpenCode::open(
        OpenOptions::default()
            .command(&cmd)
            .project_path(PROJECT_DIR)
            .chat_content(CHAT_CONTENT)
            .stream_output(true),
    )
    .await?;

    if let Some(session) = &result.session {
        println!("Session created: {}", session.id);
    }
    if let Some(reply) = &result.assistant_reply {
        let text = reply.text_content();
        if !text.is_empty() {
            println!("\nAgent reply:\n{}", text);
        }
        for part in &reply.parts {
            if part.part_type == "tool" {
                let name = part.tool_name.as_deref().unwrap_or("?");
                println!("\n[tool] {}: {:?}", name, part.tool_output.as_ref().map(|v| format!("{:?}", v).chars().take(80).collect::<String>()));
            }
        }
    }

    // Check health
    let health = result.client.health().await?;
    println!("Server version: {}", health.version);

    println!("\nDone. Check {} for the generated Python file.", PROJECT_DIR);

    if let Some(server) = result.server {
        println!("Shutting down OpenCode server (PID: {})...", server.pid());
        server.shutdown();
    }

    Ok(())
}
