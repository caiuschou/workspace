//! ChatZhipu example: demonstrates invoking Zhipu (智谱) GLM Chat Completions API.
//!
//! This example shows how to use `ChatZhipu` to have a simple conversation
//! with Zhipu's GLM model (e.g. glm-4-plus, glm-4-flash). It aligns with
//! LangChain's `invoke` API.
//!
//! ## Prerequisites
//!
//! Create a `.env` file in the example directory with your API key:
//!
//! ```bash
//! ZHIPU_API_KEY=your-api-key
//! ```
//!
//! Or set the `ZHIPU_API_KEY` environment variable.
//! Get your API key from https://open.bigmodel.cn/
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p chat-zhipu-example
//! cargo run -p chat-zhipu-example -- "你好，请用一句话介绍你自己"
//! ```

use langgraph::{ChatZhipu, LlmClient, Message};
use std::env;

#[tokio::main]
async fn main() {
    // Load .env (ZHIPU_API_KEY) before creating client
    dotenv::dotenv().ok();

    // Get user input from command line, or use default
    let user_input = env::args()
        .nth(1)
        .unwrap_or_else(|| "你好！请用一句话介绍你自己。".to_string());

    // Build message list: system prompt + user message
    let messages = vec![
        Message::system("你是一个有帮助的助手，请简洁友好地回答。"),
        Message::user(&user_input),
    ];

    // Create ChatZhipu client (reads ZHIPU_API_KEY from .env or environment)
    let client = ChatZhipu::new("glm-4-flash");

    println!("User: {user_input}");
    println!("---");

    // Invoke the model (single-call, aligns with LangChain's invoke)
    match client.invoke(&messages).await {
        Ok(response) => {
            println!("Assistant: {}", response.content);

            if !response.tool_calls.is_empty() {
                println!("\nTool calls:");
                for tc in &response.tool_calls {
                    println!("  - {}: {}", tc.name, tc.arguments);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("\nMake sure ZHIPU_API_KEY is set (get it from https://open.bigmodel.cn/):");
            eprintln!("  Add ZHIPU_API_KEY=your-api-key to .env, or export ZHIPU_API_KEY=\"your-api-key\"");
            std::process::exit(1);
        }
    }
}
