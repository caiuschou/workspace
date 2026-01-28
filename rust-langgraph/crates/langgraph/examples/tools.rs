//! 多工具示例：Calculator、HttpRequest（mock）、FileOps、ToolChain。
//!
//! 运行：`cargo run -p langgraph --example tools`
//! 演示多工具注册、按名执行与 ToolChain 串联。

use langgraph::{
    CalculatorTool, FileOpsTool, HttpRequestTool, MockHttpFetcher, Tool, ToolChain, ToolRegistry,
};
use std::sync::Arc;

fn main() {
    let mut reg = ToolRegistry::new();
    reg.register(Box::new(CalculatorTool::new()));
    reg.register(Box::new(HttpRequestTool::new(Box::new(MockHttpFetcher::new(
        r#"{"mock":true}"#,
    )))));

    let out = reg
        .execute("calculator", serde_json::json!({"expression": "2*3+1"}))
        .unwrap();
    println!("calculator 2*3+1 => {}", out);

    let out = reg
        .execute(
            "http_request",
            serde_json::json!({"url": "https://example.com", "method": "GET"}),
        )
        .unwrap();
    println!("http_request (mock) => {}", out.get("body").unwrap_or(&serde_json::Value::Null));

    let tmp = std::env::temp_dir().join("langgraph_tools_example");
    let _ = std::fs::create_dir_all(&tmp);
    let f = tmp.join("demo.txt");
    std::fs::write(&f, "hello from tools example").unwrap();
    reg.register(Box::new(FileOpsTool::new(&tmp)));
    let out = reg
        .execute("file_ops", serde_json::json!({"operation": "read", "path": "demo.txt"}))
        .unwrap();
    println!("file_ops read demo.txt => {}", out.get("content").unwrap_or(&serde_json::Value::Null));

    let registry = Arc::new(reg);
    let chain = ToolChain::new(registry.clone(), vec!["calculator".into()]);
    let out = chain
        .execute(serde_json::json!({"expression": "10-3"}))
        .unwrap();
    println!("ToolChain(calculator) 10-3 => {}", out);

    let _ = std::fs::remove_file(&f);
    let _ = std::fs::remove_dir(&tmp);
    println!("tools example ok");
}
