# OpenCode Rust SDK 设计方案

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

## 项目结构

```
opencode-rs/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # 库入口
│   ├── client.rs           # Client 主结构
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
[package]
name = "opencode"
version = "0.1.0"
edition = "2021"

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

# SSE 支持 (用于 /event 端点)
eventsource-client = "0.11"

# 错误处理
thiserror = "1"
anyhow = "1"

[dev-dependencies]
tokio-test = "0.4"
```

## 核心 API 设计

### 类型定义 (types.rs)

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub role: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MessageResponse {
    pub info: Message,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Part {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Health {
    pub healthy: bool,
    pub version: String,
}
```

### Client 实现 (client.rs)

```rust
use crate::{config::Config, error::Result, apis::*};
use reqwest::Client as HttpClient;

#[derive(Clone)]
pub struct Opencode {
    client: HttpClient,
    base_url: String,
}

impl Opencode {
    /// 创建新的 OpenCode 客户端
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: HttpClient::new(),
            base_url: base_url.into(),
        }
    }

    /// 使用配置创建
    pub fn with_config(config: Config) -> Self {
        Self::new(format!("{}:{}",
            config.hostname.unwrap_or_else(|| "127.0.0.1".into()),
            config.port.unwrap_or(4096)
        ))
    }

    // API 模块访问器
    pub fn global(&self) -> GlobalApi<'_> { GlobalApi::new(self) }
    pub fn session(&self) -> SessionApi<'_> { SessionApi::new(self) }
    pub fn find(&self) -> FindApi<'_> { FindApi::new(self) }
    pub fn file(&self) -> FileApi<'_> { FileApi::new(self) }
    pub fn project(&self) -> ProjectApi<'_> { ProjectApi::new(self) }
    pub fn config_api(&self) -> ConfigApi<'_> { ConfigApi::new(self) }
    pub fn agent(&self) -> AgentApi<'_> { AgentApi::new(self) }
    pub fn event(&self) -> EventApi<'_> { EventApi::new(self) }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
```

### Session API (apis/session.rs)

```rust
use crate::{error::Result, types::*, Opencode};
use serde::Serialize;

pub struct SessionApi<'a> {
    client: &'a Opencode,
}

impl<'a> SessionApi<'a> {
    pub fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// 列出所有会话
    pub async fn list(&self) -> Result<Vec<Session>> {
        Ok(self.client.client
            .get(self.client.url("/session"))
            .send().await?
            .error_for_status()?
            .json().await?)
    }

    /// 获取会话详情
    pub async fn get(&self, id: &str) -> Result<Session> {
        Ok(self.client.client
            .get(self.client.url(&format!("/session/{}", id)))
            .send().await?
            .error_for_status()?
            .json().await?)
    }

    /// 创建会话
    pub async fn create(&self, opts: CreateSessionOptions) -> Result<Session> {
        Ok(self.client.client
            .post(self.client.url("/session"))
            .json(&opts)
            .send().await?
            .error_for_status()?
            .json().await?)
    }

    /// 发送消息
    pub async fn send_message(
        &self,
        id: &str,
        opts: SendMessageOptions,
    ) -> Result<MessageResponse> {
        Ok(self.client.client
            .post(self.client.url(&format!("/session/{}/message", id)))
            .json(&opts)
            .send().await?
            .error_for_status()?
            .json().await?)
    }

    /// 删除会话
    pub async fn delete(&self, id: &str) -> Result<bool> {
        let status = self.client.client
            .delete(self.client.url(&format!("/session/{}", id)))
            .send().await?
            .error_for_status()?
            .status();
        Ok(status.is_success())
    }

    /// 中止会话
    pub async fn abort(&self, id: &str) -> Result<bool> {
        let status = self.client.client
            .post(self.client.url(&format!("/session/{}/abort", id)))
            .send().await?
            .error_for_status()?
            .status();
        Ok(status.is_success())
    }
}

#[derive(Debug, Serialize, Default)]
pub struct CreateSessionOptions {
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct SendMessageOptions {
    #[serde(rename = "messageId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<Model>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(rename = "noReply", skip_serializing_if = "Option::is_none")]
    pub no_reply: Option<bool>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
pub struct Model {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
}
```

### Find/File API (apis/file.rs)

```rust
use crate::{error::Result, types::*, Opencode};

pub struct FindApi<'a> { client: &'a Opencode }
pub struct FileApi<'a> { client: &'a Opencode }

impl<'a> FindApi<'a> {
    pub fn new(client: &'a Opencode) -> Self { Self { client } }

    /// 搜索文件中的文本
    pub async fn text(&self, pattern: &str) -> Result<Vec<TextMatch>> {
        Ok(self.client.client
            .get(self.client.url("/find"))
            .query(&[("pattern", pattern)])
            .send().await?
            .error_for_status()?
            .json().await?)
    }

    /// 按名称查找文件
    pub async fn files(
        &self,
        query: &str,
        r#type: Option<&str>,
        directory: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<String>> {
        let mut req = self.client.client
            .get(self.client.url("/find/file"))
            .query(&[("query", query)]);

        if let Some(t) = r#type { req = req.query(&[("type", t)]); }
        if let Some(d) = directory { req = req.query(&[("directory", d)]); }
        if let Some(l) = limit { req = req.query(&[("limit", &l.to_string())]); }

        Ok(req.send().await?.error_for_status()?.json().await?)
    }

    /// 查找工作区符号
    pub async fn symbols(&self, query: &str) -> Result<Vec<Symbol>> {
        Ok(self.client.client
            .get(self.client.url("/find/symbol"))
            .query(&[("query", query)])
            .send().await?
            .error_for_status()?
            .json().await?)
    }
}

impl<'a> FileApi<'a> {
    pub fn new(client: &'a Opencode) -> Self { Self { client } }

    /// 读取文件内容
    pub async fn read(&self, path: &str) -> Result<FileContent> {
        Ok(self.client.client
            .get(self.client.url("/file/content"))
            .query(&[("path", path)])
            .send().await?
            .error_for_status()?
            .json().await?)
    }

    /// 获取文件状态
    pub async fn status(&self) -> Result<Vec<File>> {
        Ok(self.client.client
            .get(self.client.url("/file/status"))
            .send().await?
            .error_for_status()?
            .json().await?)
    }
}
```

### SSE 事件流 (apis/event.rs)

```rust
use crate::{error::Result, Opencode};
use eventsource_client as es;
use futures::Stream;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}

pub struct EventApi<'a> { client: &'a Opencode }

impl<'a> EventApi<'a> {
    pub fn new(client: &'a Opencode) -> Self { Self { client } }

    pub async fn subscribe(&self) -> Result<impl Stream<Item = Result<Event>>> {
        let client = es::ClientBuilder::for_url(self.client.url("/event").as_str())?
            .build();

        Ok(client.stream().map(|result| match result {
            Ok(event) => {
                serde_json::from_str::<Event>(&event.data)
                    .map_err(|e| Error::InvalidResponse(e.to_string()))
            }
            Err(e) => Err(Error::InvalidResponse(e.to_string())),
        }))
    }
}
```

## 错误处理

```rust
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("JSON deserialize error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
```

## 使用示例

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
    println!("Created session: {}", session.id);

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

    if let Some(part) = response.parts.first() {
        println!("AI response: {}", part.text.as_ref().unwrap_or(&"(no text)".into()));
    }

    // 列出文件
    let files = client.find().files("*.rs", None, None, None).await?;
    println!("Found {} Rust files", files.len());

    Ok(())
}
```

## 类型生成方案

| 方案 | 工具 | 优点 | 缺点 |
|------|------|------|------|
| **A. 手写 + serde** | - | 完全控制，简洁 | 维护成本高 |
| **B. openapi-generator** | `openapi-generator-cli` | 自动生成 | 代码冗长 |
| **C. 自定义脚本** | 从 `/doc` 端点获取 | 可定制 | 需维护 |

**推荐**: 先手写核心类型，后期如果 API 变更频繁再考虑代码生成。

## API 端点映射

| SDK 方法 | HTTP 端点 | 说明 |
|----------|-----------|------|
| `global().health()` | `GET /global/health` | 健康检查 |
| `session().list()` | `GET /session` | 列出会话 |
| `session().get(id)` | `GET /session/:id` | 获取会话 |
| `session().create(opts)` | `POST /session` | 创建会话 |
| `session().send_message(id, opts)` | `POST /session/:id/message` | 发送消息 |
| `session().delete(id)` | `DELETE /session/:id` | 删除会话 |
| `session().abort(id)` | `POST /session/:id/abort` | 中止会话 |
| `find().text(pattern)` | `GET /find?pattern=` | 搜索文本 |
| `find().files(query, ...)` | `GET /find/file?query=` | 查找文件 |
| `find().symbols(query)` | `GET /find/symbol?query=` | 查找符号 |
| `file().read(path)` | `GET /file/content?path=` | 读取文件 |
| `file().status()` | `GET /file/status` | 文件状态 |
| `project().list()` | `GET /project` | 列出项目 |
| `project().current()` | `GET /project/current` | 当前项目 |
| `agent().list()` | `GET /agent` | 列出代理 |
| `config_api().get()` | `GET /config` | 获取配置 |
| `event().subscribe()` | `GET /event` | SSE 事件流 |

## 参考

- [OpenCode SDK (TypeScript)](https://opencode.ai/docs/sdk/)
- [OpenCode Server API](https://opencodecn.com/docs/server)
- [reqwest](https://docs.rs/reqwest/)
- [serde](https://serde.rs/)
