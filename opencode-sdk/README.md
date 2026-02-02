# opencode-sdk

Rust SDK for [OpenCode Server](https://opencodecn.com/docs/server) API.

## 快速开始

```rust
use opencode_sdk::{OpenCode, OpenOptions};

#[tokio::main]
async fn main() -> Result<(), opencode_sdk::Error> {
    let result = OpenCode::open(
        OpenOptions::default()
            .project_path("/path/to/your/project")
            .chat_content("你的需求，例如：分析这段代码的结构"),
    )
    .await?;

    if let Some(session) = result.session {
        println!("Session: {}", session.id);
    }
    Ok(())
}
```

## 流式输出

设置 `.stream_output(true)` 时，agent 的文本会实时打印到 stdout；`OpenResult::assistant_reply` 在整轮结束后一次性返回完整回复。

## 如何拿到 agent 的回复

`OpenResult::assistant_reply` 是 `Option<MessageListItem>`，即最后一轮 assistant 的整条消息。

- **纯文本**：`reply.text_content()` 返回各 part 的 `text` 拼接（不含 reasoning 等）。
- **逐 part 处理**：遍历 `reply.parts`，按 `part.part_type` 区分：
  - `"text"`：`part.text` 为文本内容。
  - 工具相关（如 `"tool"`、`"tool_call"`、`"tool_result"`）：用 `part.tool_name`、`part.tool_input`、`part.tool_output` 取工具调用与结果。

示例：

```rust
if let Some(reply) = &result.assistant_reply {
    let text = reply.text_content();
    if !text.is_empty() {
        println!("Agent reply:\n{}", text);
    }
    for part in &reply.parts {
        if part.part_type == "tool" {
            let name = part.tool_name.as_deref().unwrap_or("?");
            println!("[tool] {}: {:?}", name, part.tool_output);
        }
    }
}
```

## 示例与开发

完整示例（流式、session_diff、file_list 等）：`examples/cli_fibonacci.rs`，运行：`cargo run --example cli_fibonacci`。

```bash
cd opencode-sdk
cargo build
cargo test
```
