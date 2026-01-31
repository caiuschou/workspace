# opencode-sdk

Rust SDK for [OpenCode Server](https://opencodecn.com/docs/server) API.

独立 workspace，不依赖主仓库的 workspace 配置。

## 快速开始

### 日志

使用前调用 `init_logger`，日志会同时输出到控制台和文件（默认 `~/.local/share/opencode-sdk/opencode-sdk.log`）：

```rust
// 返回值 guard 需持有以保持文件日志；若不关心可忽略
let _guard = opencode_sdk::init_logger(None);  // 或 init_logger(Some(path))
```

设置 `RUST_LOG=opencode_sdk=debug` 可查看详细日志。

### 连接已有 serve 或自动启动

```rust
use opencode_sdk::{OpenCode, OpenOptions};

#[tokio::main]
async fn main() -> Result<(), opencode_sdk::Error> {
    // 若 serve 未运行，自动启动 opencode serve
    let result = OpenCode::open(OpenOptions::default()).await?;
    let health = result.client.health().await?;
    println!("Server version: {}", health.version);
    if let Some(s) = result.server {
        s.shutdown();  // 若由本 SDK 启动，可关闭
    }
    Ok(())
}
```

### 指定项目地址和聊天内容

```rust
use opencode_sdk::{OpenCode, OpenOptions};

// 在指定项目目录启动 serve，并发送初始消息
let result = OpenCode::open(
    OpenOptions::default()
        .project_path("/path/to/your/project")
        .chat_content("分析这段代码的结构")
).await?;

// result.session 为创建并已发送消息的会话
if let Some(session) = result.session {
    println!("Session: {}", session.id);
}
```

### 仅连接已有 serve

```rust
use opencode_sdk::{OpenCode, OpenOptions};

let result = OpenCode::open(
    OpenOptions::default().auto_start(false)
).await?;
```

### 流式输出

设置 `stream_output(true)` 可实时流式输出 assistant 的回复（通过订阅 GET /event SSE）：

```rust
let result = OpenCode::open(
    OpenOptions::default()
        .chat_content("写一个斐波那契")
        .stream_output(true)
).await?;
```

### 获取 Agent 回复

`OpenResult::assistant_reply` 包含 agent 的完整回复：

```rust
if let Some(reply) = &result.assistant_reply {
    // 文本内容
    println!("{}", reply.text_content());

    // 完整 parts（文本、工具调用等）
    for part in &reply.parts {
        match part.part_type.as_str() {
            "text" => println!("Text: {}", part.text.as_deref().unwrap_or("")),
            "tool" => println!("Tool: {:?}", part.tool_name),
            _ => {}
        }
    }
}
```

获取完整对话历史：`client.session_list_messages(&session.id, directory)`。

### 等待 AI 结果

当设置 `chat_content` 时，SDK 会等待 AI 完成响应后再返回（默认最多 5 分钟）：

```rust
// 自定义等待时间
let result = OpenCode::open(
    OpenOptions::default()
        .chat_content("分析代码")
        .wait_for_response_ms(60_000)  // 1 分钟
).await?;
```

### 自动安装

当 opencode 未找到时，SDK 会尝试自动安装（npm → brew → curl 脚本）：

```rust
// 默认 auto_install=true
let result = OpenCode::open(OpenOptions::default()).await?;

// 禁用自动安装
let result = OpenCode::open(
    OpenOptions::default().auto_install(false)
).await?;
```

### 直接指定 URL

```rust
use opencode_sdk::Client;

let client = Client::new("http://127.0.0.1:4096");
let health = client.health().await?;
```

## 开发

```bash
cd opencode-sdk
cargo build
cargo test
```
