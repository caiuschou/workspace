use gitlab_mcp_client::commands;

#[tokio::main]
async fn main() {
    if let Err(e) = commands::execute().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
