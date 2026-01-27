//! 流式 Chat 示例：按 token 打印，并演示多轮对话带会话记忆。
//!
//! 运行：
//! - 仅流式：`cargo run -p langgraph --example chat_stream -- "你好"`
//! - 多轮（带记忆）：`cargo run -p langgraph --example chat_stream -- --multi "第一句" "第二句"`

use std::sync::Arc;

use futures::StreamExt;
use langgraph::{ChatAgent, ChatStreamEvent, MockLlmClient, SessionMemory, StreamAgent};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let multi = args.iter().any(|a| a == "--multi");
    let inputs: Vec<&str> = if multi {
        args.iter()
            .skip_while(|a| *a != "--multi")
            .skip(1)
            .map(String::as_str)
            .collect()
    } else {
        args.get(1).map(|s| s.as_str()).into_iter().collect()
    };

    let llm = MockLlmClient::echo();
    let agent: ChatAgent<_> = if multi && inputs.len() > 1 {
        let memory = Arc::new(SessionMemory::with_capacity(32));
        ChatAgent::new(llm).with_memory(memory)
    } else {
        ChatAgent::new(llm)
    };

    for (i, input) in inputs.iter().enumerate() {
        if input.is_empty() {
            continue;
        }
        if multi && inputs.len() > 1 {
            eprintln!("--- 第 {} 轮: {} ---", i + 1, input);
        }
        let mut stream = agent.run_stream((*input).to_string());
        let mut full = String::new();
        while let Some(item) = stream.next().await {
            match item {
                Ok(ChatStreamEvent::Token(t)) => {
                    print!("{t}");
                    full.push_str(&t);
                }
                Ok(ChatStreamEvent::Done(d)) => {
                    if full.is_empty() {
                        print!("{d}");
                    }
                    println!();
                }
                Ok(ChatStreamEvent::Error(e)) => {
                    eprintln!("stream error: {e}");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
