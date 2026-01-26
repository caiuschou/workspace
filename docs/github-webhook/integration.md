# 路由集成

> [返回目录](README.md)

## 1. 路由集成

```rust
// src/webhooks/mod.rs

pub mod github;

use axum::Router;
use github::{handler::WebhookState, handlers};

use crate::config::AppConfig;

pub fn create_webhook_router(config: &AppConfig) -> Router {
    let webhook_secret = config.webhook.github.secret.clone();
    let dispatcher = handlers::create_dispatcher();
    let idempotency = github::idempotency::IdempotencyChecker::new(
        std::time::Duration::from_secs(3600), // 1 hour TTL
    );

    let state = github::handler::WebhookState {
        webhook_secret,
        dispatcher,
        idempotency,
    };

    Router::new()
        .route("/github", axum::routing::post(github::handler::github_webhook_handler))
        .route("/github/health", axum::routing::get(github::handler::health_check))
        .with_state(state)
}
```

## 2. 主应用集成

```rust
// src/lib.rs 或 src/main.rs

use axum::Router;

mod config;
mod error;
mod webhooks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load_config()?;

    let app = Router::new()
        .nest("/webhooks", webhooks::create_webhook_router(&config))
        // ... 其他路由
        ;

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&config.server.addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## 3. 配置管理

在通用配置基础上添加 Webhook 配置：

```rust
// src/config.rs

use secrecy::{Secret, SecretString};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub webhook: WebhookConfig,
    // ... 其他配置
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub addr: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WebhookConfig {
    pub github: GitHubWebhookConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubWebhookConfig {
    /// GitHub Webhook 密钥
    pub secret: SecretString,
}
```

## 4. 环境变量

```bash
# .env
SERVER_ADDR=0.0.0.0:3000
GITHUB_WEBHOOK_SECRET=your_webhook_secret_here
```

## 5. GitHub Webhook 配置步骤

1. **在 GitHub 仓库设置 Webhook**
   ```
   Settings → Webhooks → Add webhook
   ```

2. **配置参数**

| 参数 | 值 |
|------|-----|
| Payload URL | `https://your-domain.com/webhooks/github` |
| Content type | `application/json` |
| Secret | 与 `GITHUB_WEBHOOK_SECRET` 相同 |
| SSL verification | ✅ 启用 |

3. **选择事件**

选择需要监听的 GitHub 事件：
- Pushes
- Pull requests
- Issues
- ...

4. **测试**

点击 "Send me everything" 或选择具体事件后，GitHub 会发送一个 `ping` 事件来验证配置。

## 6. 完整依赖

```toml
# crates/workspace-api/Cargo.toml

[dependencies]
# Web 框架
axum = "0.7"
tokio = { version = "1", features = ["full"] }
bytes = "1"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 错误处理
thiserror = "1"
anyhow = "1"

# 安全
secrecy = { version = "0.8", features = ["serde"] }

# Webhook 签名验证
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"

# 异步 trait
async-trait = "0.1"

# 可观测性
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
prometheus = "0.13"

# 配置
dotenvy = "0.15"
```
