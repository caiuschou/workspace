# OpenCode Rust SDK

> OpenCode SDK 的 Rust 语言实现

## 概述

OpenCode Server 提供基于 OpenAPI 3.1 规范的 REST API。本 SDK 为 Rust 生态提供类型安全的 HTTP 客户端。

```
┌──────────────────────────────────────────────────────────┐
│                    opencode-rs                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │              Client (type-safe API)             │    │
│  │  - client.session.create()                      │    │
│  │  - client.find.text()                           │    │
│  │  - client.file.read()                           │    │
│  └─────────────────┬───────────────────────────────┘    │
│                    │ reqwest + HTTP                      │
└────────────────────┼──────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│              OpenCode Server (Go) :4096                  │
└──────────────────────────────────────────────────────────┘
```

## 快速开始

### 安装

```toml
[dependencies]
opencode = "0.1"
```

### 基础使用

```rust
use opencode::{Opencode, CreateSessionOptions, Part};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到本地 OpenCode Server
    let client = Opencode::new("http://127.0.0.1:4096");

    // 检查健康状态
    let health = client.global().health().await?;
    println!("Server version: {}", health.version);

    // 创建会话
    let session = client.session()
        .create(CreateSessionOptions {
            title: Some("My Rust Session".into()),
            ..Default::default()
        })
        .await?;

    // 发送消息
    let response = client.session().send_message(
        &session.id,
        opencode::SendMessageOptions {
            parts: vec![Part {
                part_type: "text".into(),
                text: Some("Hello from Rust!".into()),
            }],
            ..Default::default()
        },
    ).await?;

    println!("AI response: {}", response.parts[0].text.as_ref().unwrap());

    Ok(())
}
```

## 文档

| 文档 | 描述 |
|------|------|
| [核心概念](concepts.md) | OpenCode 核心概念说明 |
| [架构设计](architecture.md) | 架构索引；详见 [architecture/](architecture/README.md) 子文档 |
| [OpenCode::open 设计](open-method-design.md) | 自动启动 serve 的 `open` 方法设计 |
| [API 参考](api-reference.md) | 完整 API 文档 |
| [类型定义](types.md) | 所有数据类型 |
| [示例代码](examples.md) | 更多使用示例 |

## 项目结构

```
opencode-rs/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # 库入口
│   ├── client/               # Client 主结构
│   │   ├── mod.rs            # Client, HealthResponse
│   │   └── builder.rs        # ClientBuilder
│   ├── config.rs           # 配置
│   ├── error.rs            # 错误类型
│   └── apis/
│       ├── mod.rs
│       ├── global.rs       # /global/*
│       ├── session.rs      # /session/*
│       ├── file.rs         # /file, /find/*
│       ├── project.rs      # /project/*
│       ├── config_api.rs   # /config/*
│       ├── agent.rs        # /agent/*
│       └── event.rs        # /event (SSE)
├── openapi/
│   └── openapi.json        # 从服务器获取的 OpenAPI 规范
└── examples/
    └── basic.rs
```

## 依赖

```toml
[dependencies]
# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "stream"] }

# 异步运行时
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# SSE 支持
eventsource-client = "0.11"

# 错误处理
thiserror = "1"
anyhow = "1"
```

## 参考

- [OpenCode SDK (TypeScript)](https://opencode.ai/docs/sdk/)
- [OpenCode Server API](https://opencodecn.com/docs/server)
- [reqwest](https://docs.rs/reqwest/)
- [serde](https://serde.rs/)
