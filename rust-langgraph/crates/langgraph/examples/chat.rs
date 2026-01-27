//! Chat 示例：从命令行参数取一问，经 ChatAgent（Mock LLM）得一答并打印。
//!
//! 运行：`cargo run -p langgraph --example chat -- "你好"`
//! 使用 Mock 时无需 API Key；若接入真实 OpenAI，需配置相应环境变量（见 ROADMAP S2）。

use langgraph::{AsyncAgent, ChatAgent, MockLlmClient};
use std::env;

#[tokio::main]
async fn main() {
    let input = env::args().nth(1).unwrap_or_else(|| "".to_string());

    let llm = MockLlmClient::echo();
    let agent: ChatAgent<_> = ChatAgent::new(llm);

    match agent.run(input).await {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
