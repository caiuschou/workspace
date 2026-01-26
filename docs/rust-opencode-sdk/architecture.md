# 架构设计

## 设计原则

1. **类型安全** - 所有 API 请求/响应都有强类型定义
2. **异步优先** - 基于 `tokio` 和 `reqwest` 的异步实现
3. **模块化** - 按功能域划分 API 模块
4. **可扩展** - 易于添加新的 API 端点

## 项目结构

```
opencode-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs              # 库入口，导出公共 API
│   ├── client.rs           # Client 主结构
│   ├── config.rs           # 配置相关
│   ├── error.rs            # 错误类型定义
│   ├── types.rs            # 共享数据类型
│   └── apis/
│       ├── mod.rs          # API 模块入口
│       ├── global.rs       # GET /global/health
│       ├── session.rs      # GET/POST/DELETE /session/*
│       ├── file.rs         # /file, /find/*
│       ├── project.rs      # /project/*
│       ├── config_api.rs   # /config/*
│       ├── agent.rs        # /agent/*
│       ├── provider.rs     # /provider/*
│       ├── command.rs      # /command/*
│       ├── auth.rs         # /auth/*
│       ├── tui.rs          # /tui/*
│       └── event.rs        # /event (SSE)
└── examples/
```

## 核心组件

### 1. Client

```rust
use reqwest::Client as HttpClient;

#[derive(Clone)]
pub struct Opencode {
    client: HttpClient,
    base_url: String,
}

impl Opencode {
    pub fn new(base_url: impl Into<String>) -> Self { ... }
    pub fn with_config(config: Config) -> Self { ... }

    // API 访问器
    pub fn global(&self) -> GlobalApi<'_> { ... }
    pub fn session(&self) -> SessionApi<'_> { ... }
    pub fn find(&self) -> FindApi<'_> { ... }
    pub fn file(&self) -> FileApi<'_> { ... }
    pub fn project(&self) -> ProjectApi<'_> { ... }
    pub fn config_api(&self) -> ConfigApi<'_> { ... }
    pub fn agent(&self) -> AgentApi<'_> { ... }
    pub fn event(&self) -> EventApi<'_> { ... }
}
```

### 2. API 模块

每个 API 模块遵循统一模式：

```rust
pub struct XxxApi<'a> {
    client: &'a Opencode,
}

impl<'a> XxxApi<'a> {
    pub fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    // CRUD 方法
    pub async fn list(&self) -> Result<Vec<Xxx>> { ... }
    pub async fn get(&self, id: &str) -> Result<Xxx> { ... }
    pub async fn create(&self, opts: CreateXxxOptions) -> Result<Xxx> { ... }
    pub async fn delete(&self, id: &str) -> Result<bool> { ... }
}
```

### 3. 错误处理

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

### 4. 配置

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub timeout: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hostname: Some("127.0.0.1".into()),
            port: Some(4096),
            timeout: Some(Duration::from_secs(30)),
        }
    }
}
```

## 设计模式

### Builder 模式

用于复杂的请求选项：

```rust
pub struct SendMessageOptions {
    pub message_id: Option<String>,
    pub model: Option<Model>,
    pub agent: Option<String>,
    pub no_reply: Option<bool>,
    pub parts: Vec<Part>,
}

impl SendMessageOptions {
    pub fn builder() -> SendMessageOptionsBuilder {
        SendMessageOptionsBuilder::default()
    }
}
```

### Fluent API

链式调用风格：

```rust
let response = client
    .session()
    .send_message(&session_id, opts)
    .await?;
```

## 生命周期设计

API 模块使用引用生命周期，避免克隆 Client：

```rust
pub struct SessionApi<'a> {
    client: &'a Opencode,  // 借用，不拥有
}
```

这允许：
```rust
let client = Opencode::new(...);
let session_api = client.session();  // 不消耗 client
let global_api = client.global();     // 可以多次调用
```

## 异步设计

所有 API 方法都是异步的，使用 `async/await`：

```rust
pub async fn create(&self, opts: CreateSessionOptions) -> Result<Session> {
    let response = self.client.client
        .post(self.client.url("/session"))
        .json(&opts)
        .send()
        .await?;  // 异步等待
    // ...
}
```

## SSE 事件流

使用 `futures::Stream` 处理服务器推送事件：

```rust
pub async fn subscribe(&self) -> Result<impl Stream<Item = Result<Event>>> {
    let client = eventsource_client::ClientBuilder::for_url(...)
        .build()?;

    Ok(client.stream().map(|result| { ... }))
}
```

使用方式：

```rust
let mut stream = client.event().subscribe().await?;
while let Some(event) = stream.next().await {
    match event {
        Ok(e) => println!("Event: {}", e.event_type),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## 类型生成策略

| 方案 | 工具 | 优点 | 缺点 |
|------|------|------|------|
| **手写 + serde** | - | 完全控制，代码简洁 | 维护成本高 |
| **openapi-generator** | CLI | 自动生成 | 代码冗长，难以定制 |
| **自定义脚本** | 从 `/doc` 端点获取 | 可定制 | 需要维护脚本 |

**推荐**: 核心类型手写，后期可从 OpenAPI 规范自动生成。
