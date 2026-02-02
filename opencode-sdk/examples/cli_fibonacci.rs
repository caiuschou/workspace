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
    // Logging: guard must be held so that file logging stays active.
    let _guard = init_logger(None);

    std::fs::create_dir_all(PROJECT_DIR).expect("create project dir");

    let cmd = opencode_command();
    println!("Opening OpenCode...");
    println!("  command: {}", cmd);
    println!("  project: {}", PROJECT_DIR);
    println!("  prompt:  {}", CHAT_CONTENT);

    // Open: connect or start serve, create session, send chat_content, stream agent output to stdout,
    // then wait for completion and return with session + assistant_reply.
    let result = OpenCode::open(
        OpenOptions::default()
            .command(&cmd)
            .project_path(PROJECT_DIR)
            .chat_content(CHAT_CONTENT)
            .stream_output(true),  // Each SSE text delta is printed in real time.
    )
    .await?;

    // Session created when chat_content was provided.
    if let Some(session) = &result.session {
        println!("Session created: {}", session.id);
    }
    // Last assistant message (full reply after stream ended): text_content() + tool parts.
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

    let health = result.client.health().await?;
    println!("Server version: {}", health.version);

    let project_path = std::path::Path::new(PROJECT_DIR);

    // Session diff: file changes produced in this session (GET /session/{id}/diff).
    if let Some(session) = &result.session {
        println!("\n--- Session diff (GET /session/{{id}}/diff) ---");
        match result
            .client
            .session_diff(&session.id, Some(project_path), None)
            .await
        {
            Ok(diff) => {
                let pretty = serde_json::to_string_pretty(&diff).unwrap_or_else(|_| format!("{:?}", diff));
                println!("{}", pretty);
            }
            Err(e) => println!("Session diff error: {}", e),
        }
    }

    // File list: files/dirs under project root (GET /file?path=.).
    println!("\n--- File list (GET /file?path=.) ---");
    match result.client.file_list(".", Some(project_path)).await {
        Ok(list) => {
            let pretty = serde_json::to_string_pretty(&list).unwrap_or_else(|_| format!("{:?}", list));
            println!("{}", pretty);
        }
        Err(e) => println!("File list error: {}", e),
    }

    // File status: git status in project (GET /file/status).
    println!("\n--- File status (GET /file/status) ---");
    match result.client.file_status(Some(project_path)).await {
        Ok(status) => {
            let pretty = serde_json::to_string_pretty(&status).unwrap_or_else(|_| format!("{:?}", status));
            println!("{}", pretty);
        }
        Err(e) => println!("File status error: {}", e),
    }

    println!("\nDone. Check {} for the generated Python file.", PROJECT_DIR);

    // If we started the server, shut it down.
    if let Some(server) = result.server {
        println!("Shutting down OpenCode server (PID: {})...", server.pid());
        server.shutdown();
    }

    // Avoid blocking on WorkerGuard drop: with trace level, flushing to file can take long.
    std::mem::forget(_guard);

    Ok(())
}
