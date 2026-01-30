//! ChatOpenAI example: demonstrates invoking OpenAI Chat Completions API.
//!
//! This example shows how to use `ChatOpenAI` to have a simple conversation
//! with OpenAI's GPT model. It aligns with LangChain's `invoke` API.
//!
//! ## Prerequisites
//!
//! Set the `OPENAI_API_KEY` environment variable before running:
//!
//! ```bash
//! export OPENAI_API_KEY="sk-..."
//! ```
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p chat-openai-example
//! cargo run -p chat-openai-example -- "What is Rust?"
//! ```

use langgraph::{ChatOpenAI, LlmClient, Message};
use std::env;

#[tokio::main]
async fn main() {
    // Get user input from command line, or use default
    let user_input = env::args()
        .nth(1)
        .unwrap_or_else(|| "Hello! Please introduce yourself in one sentence.".to_string());

    // Build message list: system prompt + user message
    let messages = vec![
        Message::system("You are a helpful assistant. Be concise and friendly."),
        Message::user(&user_input),
    ];

    // Create ChatOpenAI client (reads OPENAI_API_KEY from environment)
    let client = ChatOpenAI::new("gpt-4o-mini");

    println!("User: {user_input}");
    println!("---");

    // Invoke the model (single-call, aligns with LangChain's invoke)
    match client.invoke(&messages).await {
        Ok(response) => {
            println!("Assistant: {}", response.content);

            // Show tool_calls if any (usually empty without with_tools)
            if !response.tool_calls.is_empty() {
                println!("\nTool calls:");
                for tc in &response.tool_calls {
                    println!("  - {}: {}", tc.name, tc.arguments);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("\nMake sure OPENAI_API_KEY is set:");
            eprintln!("  export OPENAI_API_KEY=\"sk-...\"");
            std::process::exit(1);
        }
    }
}
