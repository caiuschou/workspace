# Workspace API 架构设计

## 概述

本文档描述 `workspace-api` 的通用架构设计，包括配置管理、错误处理、可观测性等基础设施。

## 1. 项目结构

```
crates/workspace-api/src/
├── main.rs                      # 应用入口
├── lib.rs                       # 库入口，路由集成
├── config.rs                    # 配置管理
├── error.rs                     # 通用错误类型
├── metrics.rs                   # 可观测性
└── webhooks/                    # Webhook 模块
    └── github/                  # GitHub Webhook (见独立文档)
```

## 2. 依赖配置

```toml
# crates/workspace-api/Cargo.toml

[dependencies]
# Web 框架
axum = "0.7"
tokio = { workspace = true, features = ["full", "sync"] }

# 序列化
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }

# 日志
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# HTTP 中间件
tower-http = { version = "0.5", features = ["trace", "limit", "cors"] }
tower-governor = "0.3"           # 速率限制

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 安全
secrecy = "0.8"                  # 敏感数据包装

# 可观测性
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

# 配置管理
config = "0.13"
```

## 3. 配置管理

### 3.1 配置结构

```rust
// src/config.rs

use secrecy::SecretString;
use std::collections::HashMap;
use std::time::Duration;

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 请求体大小限制
    pub max_body_size: usize,
    /// 请求超时
    pub request_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 3000,
            max_body_size: 10 * 1024 * 1024, // 10MB
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// Webhook 配置
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// GitHub Webhook Secret
    pub github_secret: Option<SecretString>,
    /// 仓库特定 secret: "owner/repo" -> secret
    pub repo_secrets: HashMap<String, SecretString>,
    /// 是否启用幂等性检查
    pub idempotency_enabled: bool,
    /// 幂等键 TTL
    pub idempotency_ttl: Duration,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            github_secret: None,
            repo_secrets: HashMap::new(),
            idempotency_enabled: true,
            idempotency_ttl: Duration::from_secs(86400), // 24h
        }
    }
}

impl WebhookConfig {
    /// 获取指定仓库的 secret
    pub fn get_secret(&self, repo: &str) -> Option<&SecretString> {
        self.repo_secrets.get(repo).or(self.github_secret.as_ref())
    }
}

/// 应用配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub webhook: WebhookConfig,
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, ConfigError> {
        let github_secret = std::env::var("GITHUB_WEBHOOK_SECRET")
            .ok()
            .filter(|s| !s.is_empty())
            .map(SecretString::new);

        Ok(Self {
            server: ServerConfig {
                host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
                port: std::env::var("PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3000),
                ..Default::default()
            },
            webhook: WebhookConfig {
                github_secret,
                ..Default::default()
            },
        })
    }
}

/// 配置错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required config: {0}")]
    Missing(&'static str),

    #[error("Invalid config value: {0}")]
    Invalid(String),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] std::env::VarError),
}
```

### 3.2 环境变量

| 变量名 | 描述 | 默认值 | 必需 |
|--------|------|--------|------|
| `HOST` | 监听地址 | `0.0.0.0` | 否 |
| `PORT` | 监听端口 | `3000` | 否 |
| `RUST_LOG` | 日志级别 | `workspace_api=debug` | 否 |
| `GITHUB_WEBHOOK_SECRET` | GitHub Webhook 密钥 | - | 推荐 |

## 4. 错误处理

### 4.1 错误类型定义

```rust
// src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

/// API 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// 应用错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // 客户端错误 (4xx)
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Too many requests")]
    TooManyRequests,

    // 服务端错误 (5xx)
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
            Self::TooManyRequests => "TOO_MANY_REQUESTS",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // 内部错误不暴露详情
        let message = match &self {
            Self::Internal(e) => {
                tracing::error!(error = %e, "Internal error");
                "An internal error occurred".to_string()
            }
            _ => self.to_string(),
        };

        let body = ErrorResponse {
            error: self.error_code(),
            message,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

/// 便捷类型别名
pub type AppResult<T> = Result<T, AppError>;
```

### 4.2 错误处理最佳实践

```rust
// 使用示例

// 1. 从 anyhow::Error 自动转换
async fn some_handler() -> AppResult<Json<Data>> {
    let data = fetch_data().await?; // anyhow::Error -> AppError::Internal
    Ok(Json(data))
}

// 2. 显式构造错误
async fn get_user(id: u64) -> AppResult<Json<User>> {
    let user = db.find_user(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;
    Ok(Json(user))
}

// 3. 使用 anyhow::Context 添加上下文
use anyhow::Context;

async fn process() -> AppResult<()> {
    do_something()
        .await
        .context("Failed to do something")?;
    Ok(())
}
```

## 5. 可观测性

### 5.1 Metrics 定义

```rust
// src/metrics.rs

use metrics::{counter, histogram, gauge, describe_counter, describe_histogram, describe_gauge};
use std::time::Instant;

/// 初始化 metrics 描述
pub fn init_metrics() {
    // HTTP 请求指标
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    );
    describe_gauge!(
        "http_requests_in_flight",
        "Number of HTTP requests currently being processed"
    );

    // 应用指标
    describe_counter!(
        "app_errors_total",
        "Total number of application errors"
    );
}

/// HTTP 请求指标记录器
pub struct RequestMetrics {
    path: String,
    method: String,
    start: Instant,
}

impl RequestMetrics {
    pub fn new(path: &str, method: &str) -> Self {
        gauge!("http_requests_in_flight").increment(1.0);
        Self {
            path: path.to_string(),
            method: method.to_string(),
            start: Instant::now(),
        }
    }

    pub fn finish(self, status: u16) {
        let duration = self.start.elapsed().as_secs_f64();
        let status_class = format!("{}xx", status / 100);

        counter!(
            "http_requests_total",
            "path" => self.path.clone(),
            "method" => self.method.clone(),
            "status" => status_class.clone()
        ).increment(1);

        histogram!(
            "http_request_duration_seconds",
            "path" => self.path,
            "method" => self.method,
            "status" => status_class
        ).record(duration);

        gauge!("http_requests_in_flight").decrement(1.0);
    }
}

/// 记录应用错误
pub fn record_error(error_type: &str, component: &str) {
    counter!(
        "app_errors_total",
        "type" => error_type.to_string(),
        "component" => component.to_string()
    ).increment(1);
}
```

### 5.2 Prometheus 集成

```rust
// src/lib.rs (部分)

use metrics_exporter_prometheus::PrometheusBuilder;

pub fn setup_metrics() -> Result<PrometheusHandle, Box<dyn std::error::Error>> {
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()?;

    metrics::init_metrics();

    Ok(handle)
}

// 添加 /metrics 端点
async fn metrics_handler(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}
```

### 5.3 Prometheus 指标汇总

| 指标名 | 类型 | 标签 | 描述 |
|--------|------|------|------|
| `http_requests_total` | Counter | path, method, status | HTTP 请求总数 |
| `http_request_duration_seconds` | Histogram | path, method, status | 请求延迟 |
| `http_requests_in_flight` | Gauge | - | 当前处理中的请求数 |
| `app_errors_total` | Counter | type, component | 应用错误总数 |

### 5.4 日志配置

```rust
// src/main.rs

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "workspace_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true))
        .init();
}
```

## 6. 优雅关闭

```rust
// src/lib.rs (部分)

use tokio::signal;

/// 优雅关闭信号处理
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        },
    }
}
```

## 7. 应用入口

```rust
// src/main.rs

use workspace_api::{run_server, init_tracing};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化日志
    init_tracing();

    // 2. 启动服务
    tracing::info!("Starting workspace-api");
    run_server().await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
```

```rust
// src/lib.rs

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};

mod config;
mod error;
mod metrics;
mod webhooks;

pub use config::{AppConfig, ConfigError};
pub use error::{AppError, AppResult};

use config::AppConfig;
use webhooks::github;

/// 运行服务器
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置
    let config = Arc::new(AppConfig::from_env()?);

    // 2. 初始化 metrics
    let metrics_handle = setup_metrics()?;
    metrics::init_metrics();

    // 3. 构建应用状态
    let app_state = AppState::new(config.clone())?;

    // 4. 构建路由
    let app = Router::new()
        // 基础路由
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        // Webhook 路由 (见 github-webhook-integration.md)
        .nest("/webhooks", webhooks::router(app_state.clone()))
        // 中间件
        .layer(RequestBodyLimitLayer::new(config.server.max_body_size))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // 5. 启动服务器
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn root_handler() -> &'static str {
    "Hello from Workspace API!"
}

async fn health_handler() -> &'static str {
    "OK"
}
```

## 8. 安全最佳实践

### 8.1 敏感数据处理

```rust
use secrecy::{ExposeSecret, SecretString};

// 敏感数据使用 SecretString 包装
pub struct Credentials {
    pub api_key: SecretString,
}

// 访问时显式调用 expose_secret()
fn use_credential(cred: &Credentials) {
    let key = cred.api_key.expose_secret();
    // 使用 key...
}

// SecretString 的 Debug 实现会打印 [REDACTED]
// 防止敏感数据泄露到日志
```

### 8.2 安全检查清单

- [x] 敏感数据使用 `secrecy` crate 包装
- [x] 内部错误不暴露详情给客户端
- [x] 请求体大小限制
- [x] 速率限制 (可选启用)
- [x] 请求追踪和日志
- [ ] HTTPS (在反向代理层配置)
- [ ] CORS 配置 (按需启用)

## 9. 相关文档

- [GitHub Webhook 集成](./github-webhook-integration.md) - GitHub Webhook 接收和处理
