//! Echo 示例：从命令行参数取一句话，经 EchoAgent 回显。
//!
//! 运行：`cargo run -p langgraph --example echo -- "你好"`

use langgraph::{Agent, EchoAgent};
use std::env;

fn main() {
    let input = env::args()
        .nth(1)
        .unwrap_or_else(|| "".to_string());

    let agent = EchoAgent::new();
    match agent.run(input.clone()) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
