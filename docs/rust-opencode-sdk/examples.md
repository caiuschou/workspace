# 示例代码

## 基础使用

### 连接并检查健康状态

```rust
use opencode::Opencode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    let health = client.global().health().await?;
    println!("Server healthy: {}", health.healthy);
    println!("Server version: {}", health.version);

    Ok(())
}
```

---

## 会话管理

### 创建会话并列出

```rust
use opencode::{Opencode, CreateSessionOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    // 创建新会话
    let session = client.session()
        .create(CreateSessionOptions {
            title: Some("My Rust Session".into()),
            ..Default::default()
        })
        .await?;

    println!("Created session: {} ({})", session.title.as_ref().unwrap(), session.id);

    // 列出所有会话
    let sessions = client.session().list().await?;
    println!("Total sessions: {}", sessions.len());

    Ok(())
}
```

### 发送消息

```rust
use opencode::{Opencode, SendMessageOptions, Part, Model};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    // 假设已有会话 ID
    let session_id = "session-id-here";

    let response = client.session().send_message(
        session_id,
        SendMessageOptions {
            model: Some(Model {
                provider_id: "anthropic".into(),
                model_id: "claude-3-5-sonnet-20241022".into(),
            }),
            parts: vec![Part {
                part_type: "text".into(),
                text: Some("解释一下 Rust 的所有权机制".into()),
                ..Default::default()
            }],
            ..Default::default()
        },
    ).await?;

    // 打印 AI 回复
    for part in response.parts {
        if let Some(text) = part.text {
            println!("AI: {}", text);
        }
    }

    Ok(())
}
```

### 只注入上下文（不触发回复）

```rust
// 使用 noReply: true 仅注入上下文
client.session().send_message(session_id, SendMessageOptions {
    no_reply: Some(true),
    parts: vec![Part {
        part_type: "text".into(),
        text: Some("你是一个 Rust 专家".into()),
        ..Default::default()
    }],
    ..Default::default()
}).await?;
```

---

## 文件搜索

### 搜索文本

```rust
use opencode::Opencode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    // 在所有文件中搜索 "TODO"
    let matches = client.find().text("TODO").await?;

    for m in matches {
        println!("{}:{} - {}", m.path, m.line_number, m.lines.trim());
    }

    Ok(())
}
```

### 查找文件

```rust
// 查找所有 Rust 文件
let files = client.find()
    .files(
        "*.rs",           // query
        Some("file"),     // type
        None,             // directory
        Some(100),        // limit
    )
    .await?;

for file in files {
    println!("{}", file);
}
```

### 查找符号

```rust
// 查找函数定义
let symbols = client.find().symbols("main").await?;

for symbol in symbols {
    println!("{} - {} ({})", symbol.name, symbol.path.as_ref().unwrap(), symbol.kind.as_ref().unwrap_or(&"?".into()));
}
```

---

## 文件操作

### 读取文件

```rust
use opencode::Opencode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    let content = client.file().read("src/main.rs").await?;

    match content.content_type.as_str() {
        "raw" => println!("{}", content.content),
        "patch" => println!("Patch: {}", content.content),
        _ => {}
    }

    Ok(())
}
```

### 获取文件状态

```rust
let files = client.file().status().await?;

for file in files {
    println!("{} ({})", file.path, file.file_type);
}
```

---

## 事件流

### 监听服务器事件

```rust
use opencode::Opencode;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    let mut stream = client.event().subscribe().await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                match event.event_type.as_str() {
                    "server.connected" => println!("Connected!"),
                    "session.created" => println!("Session created: {:?}", event.properties),
                    "message.created" => println!("Message created"),
                    _ => println!("Other event: {}", event.event_type),
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

---

## 配置管理

### 获取配置

```rust
use opencode::Opencode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    // 获取配置
    let config = client.config_api().get().await?;
    println!("Default model: {:?}", config.model);

    // 获取提供商列表
    let providers = client.config_api().providers().await?;
    println!("Available providers:");
    for p in providers.providers {
        println!("  - {}", p.name);
    }
    println!("Default models: {:?}", providers.default_models);

    Ok(())
}
```

---

## 代理操作

### 列出所有代理

```rust
use opencode::Opencode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    let agents = client.agent().list().await?;

    println!("Available agents:");
    for agent in agents {
        println!("  {} - {}", agent.id, agent.name);
        if let Some(desc) = agent.description {
            println!("    {}", desc);
        }
    }

    Ok(())
}
```

---

## TUI 控制

### 控制 TUI 界面

```rust
use opencode::Opencode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Opencode::new("http://127.0.0.1:4096");

    // 追加文本到提示框
    client.tui().append_prompt("帮我检查这段代码").await?;

    // 显示通知
    client.tui().showToast("任务完成", "success").await?;

    // 打开会话选择器
    client.tui().open_sessions().await?;

    Ok(())
}
```

---

## 完整示例

### 简单的 CLI 工具

```rust
use opencode::{Opencode, CreateSessionOptions, SendMessageOptions, Part};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <prompt>", args[0]);
        return Ok(());
    }

    let prompt = &args[1];

    let client = Opencode::new("http://127.0.0.1:4096");

    // 检查连接
    let health = client.global().health().await?;
    if !health.healthy {
        println!("Server is not healthy!");
        return Ok(());
    }

    // 创建会话
    let session = client.session()
        .create(CreateSessionOptions {
            title: Some("CLI Session".into()),
            ..Default::default()
        })
        .await?;

    println!("Session: {}", session.id);

    // 发送消息
    let response = client.session().send_message(
        &session.id,
        SendMessageOptions {
            parts: vec![Part {
                part_type: "text".into(),
                text: Some(prompt.clone()),
                ..Default::default()
            }],
            ..Default::default()
        },
    ).await?;

    // 打印回复
    for part in response.parts {
        if let Some(text) = part.text {
            println!("{}", text);
        }
    }

    // 清理会话
    client.session().delete(&session.id).await?;

    Ok(())
}
```

---

## 错误处理

### 完整的错误处理

```rust
use opencode::{Opencode, Error};

#[tokio::main]
async fn main() {
    let client = Opencode::new("http://127.0.0.1:4096");

    match client.global().health().await {
        Ok(health) => {
            println!("Server version: {}", health.version);
        }
        Err(Error::Api { status, message }) => {
            eprintln!("API error {}: {}", status, message);
        }
        Err(Error::NotFound(msg)) => {
            eprintln!("Not found: {}", msg);
        }
        Err(Error::Http(e)) => {
            eprintln!("HTTP error: {}", e);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

---

## 使用配置

### 自定义配置

```rust
use opencode::{Opencode, Config};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        hostname: Some("localhost".into()),
        port: Some(4096),
        timeout: Some(Duration::from_secs(60)),
    };

    let client = Opencode::with_config(config);

    let health = client.global().health().await?;
    println!("Connected: {}", health.healthy);

    Ok(())
}
```
