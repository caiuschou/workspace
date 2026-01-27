# 生产级实现

构建可部署的生产级 Agent 应用。

## HTTP API

```rust
use axum::{
    extract::{State, Path},
    response::{IntoResponse, Json, sse::Sse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

/// API 状态
#[derive(Clone)]
pub struct ApiState {
    agents: Arc<RwLock<AgentRegistry>>,
    metrics: Arc<Metrics>,
}

/// 路由
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(health))

        // Agent 路由
        .route("/agents", get(list_agents))
        .route("/agents/:name", post(run_agent))
        .route("/agents/:name/stream", post(stream_agent))

        // 监控
        .route("/metrics", get(metrics_export))

        .with_state(state)
}

/// 健康检查
async fn health() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    version: String,
}

/// 列出 Agents
async fn list_agents(
    State(state): State<ApiState>,
) -> Result<Json<Vec<AgentInfo>>, ApiError> {
    let agents = state.agents.read().await;
    let list = agents.list().await;
    Ok(Json(list))
}

#[derive(Serialize)]
struct AgentInfo {
    name: String,
    description: String,
    status: String,
}

/// 运行 Agent
async fn run_agent(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(req): RunRequest,
) -> Result<Json<RunResponse>, ApiError> {
    let start = Instant::now();

    let agents = state.agents.read().await;
    let result = agents.run(&name, req.input).await?;

    let duration = start.elapsed();
    state.metrics.record(&name, duration, true);

    Ok(Json(RunResponse {
        result,
        duration_ms: duration.as_millis() as u64,
    }))
}

#[derive(Deserialize)]
struct RunRequest {
    input: serde_json::Value,
}

#[derive(Serialize)]
struct RunResponse {
    result: serde_json::Value,
    duration_ms: u64,
}

/// 流式运行
async fn stream_agent(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(req): RunRequest>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let agents = state.agents.read().await;
    // 启动流式任务
    let _ = agents.run_stream(&name, req.input, tx);

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
        .map(|item| Ok(SseEvent::from(item)));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keepalive"),
    )
}

#[derive(Clone, Serialize)]
struct SseEvent {
    #[serde(rename = "type")]
    kind: String,
    data: serde_json::Value,
}

impl From<StreamItem> for SseEvent {
    fn from(item: StreamItem) -> Self {
        match item {
            StreamItem::Start => Self {
                kind: "start".to_string(),
                data: serde_json::json!({}),
            },
            StreamItem::Chunk(text) => Self {
                kind: "chunk".to_string(),
                data: serde_json::json!({ "text": text }),
            },
            StreamItem::Done => Self {
                kind: "done".to_string(),
                data: serde_json::json!({}),
            },
            StreamItem::Error(e) => Self {
                kind: "error".to_string(),
                data: serde_json::json!({ "error": e.to_string() }),
            },
        }
    }
}

#[derive(Debug, Clone)]
enum StreamItem {
    Start,
    Chunk(String),
    Done,
    Error(AgentError),
}
```

## 配置管理

```rust
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub llm: LlmConfig,
    pub database: DatabaseConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    pub tracing_level: String,
    pub metrics_enabled: bool,
    pub jaeger_endpoint: Option<String>,
}

impl AppConfig {
    /// 从环境变量和配置文件加载
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            .add_source(Environment::default().separator("__"))
            .build()?;

        config.try_deserialize().map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Missing required field: {0}")]
    MissingField(String),
}
```

## 指标收集

```rust
use prometheus::{
    Counter, Histogram, IntGauge, Registry,
    opts, register_counter, register_histogram, register_int_gauge,
};

/// 应用指标
pub struct Metrics {
    /// 请求总数
    pub requests_total: Counter,

    /// 请求延迟
    pub request_duration: Histogram,

    /// 活跃连接
    pub active_connections: IntGauge,

    /// LLM token 使用
    pub llm_tokens: Counter,

    registry: Registry,
}

impl Metrics {
    pub fn new() -> Self {
        let requests_total = register_counter!(
            opts!("agent_requests_total", "Total agent requests")
            .namespace("langgraph")
        ).unwrap();

        let request_duration = register_histogram!(
            prometheus::HistogramOpts::new(
                "agent_request_duration_seconds",
                "Agent request duration"
            )
            .namespace("langgraph")
            .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        ).unwrap();

        let active_connections = register_int_gauge!(
            opts!("agent_active_connections", "Active connections")
            .namespace("langgraph")
        ).unwrap();

        let llm_tokens = register_counter!(
            opts!("agent_llm_tokens_total", "Total LLM tokens used")
            .namespace("langgraph")
        ).unwrap();

        let registry = Registry::new();

        Self {
            requests_total,
            request_duration,
            active_connections,
            llm_tokens,
            registry,
        }
    }

    /// 记录请求
    pub fn record(&self, agent: &str, duration: Duration, success: bool) {
        self.requests_total.inc();
        self.request_duration.observe(duration.as_secs_f64());
    }

    /// 导出指标
    pub fn export(&self) -> String {
        use prometheus::Encoder;

        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();

        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

/// 指标端点
async fn metrics_export(State(state): State<ApiState>) -> String {
    state.metrics.export()
}
```

## 可观测性

```rust
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// 初始化日志
pub fn init_logging(config: &ObservabilityConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.tracing_level));

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json());

    if let Some(endpoint) = &config.jaeger_endpoint {
        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_endpoint(endpoint)
            .with_service_name("langgraph-agent")
            .install_simple()
            .unwrap();

        subscriber.with(tracing_opentelemetry::layer().with_tracer(tracer)).init();
    } else {
        subscriber.init();
    }
}

/// 带追踪的 Agent 包装器
pub struct TracedAgent<A> {
    inner: A,
    name: String,
}

impl<A> TracedAgent<A> {
    pub fn new(name: String, inner: A) -> Self {
        Self { inner, name }
    }
}

#[async_trait]
impl<A: Agent> Agent for TracedAgent<A> {
    type State = A::State;
    type Input = A::Input;
    type Output = A::Output;

    #[instrument(skip(self, input))]
    async fn run(&self, input: Self::Input) -> Result<Self::Output, AgentError> {
        let span = tracing::info_span!("agent", name = %self.name);
        let _enter = span.enter();

        info!("Starting agent execution");

        let start = Instant::now();
        let result = self.inner.run(input).await;
        let duration = start.elapsed();

        match &result {
            Ok(_) => info!(duration_ms = duration.as_millis(), "Agent completed"),
            Err(e) => error!(duration_ms = duration.as_millis(), error = %e, "Agent failed"),
        }

        result
    }
}
```

## 限流和熔断

```rust
use governor::{Quota, RateLimiter, Jitter, state::InMemoryState};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// 限流器
pub struct RateLimiter {
    limiter: Arc<governor::RateLimiter<governor::state::direct::NotKeyed, InMemoryState>>,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));

        Self { limiter }
    }

    pub async fn check(&self) -> Result<(), RateLimitError> {
        self.limiter
            .check()
            .map_err(|_| RateLimitError::LimitExceeded)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    LimitExceeded,
}

/// 熔断器
pub struct CircuitBreaker<T> {
    inner: T,
    state: Arc<RwLock<CircuitState>>,
    threshold: usize,
    timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl<T> CircuitBreaker<T> {
    pub fn new(inner: T, threshold: usize, timeout: Duration) -> Self {
        Self {
            inner,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            threshold,
            timeout,
        }
    }

    pub async fn call<F, R, E>(&self, f: F) -> Result<R, CircuitBreakerError<E>>
    where
        F: FnOnce(&T) -> Result<R, E>,
    {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Open => {
                return Err(CircuitBreakerError::Open);
            }
            CircuitState::Closed | CircuitState::HalfOpen => {}
        }

        drop(state);

        match f(&self.inner) {
            Ok(result) => {
                let mut state = self.state.write().await;
                *state = CircuitState::Closed;
                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.write().await;
                *state = CircuitState::Open;

                tokio::spawn(async move {
                    tokio::time::sleep(self.timeout).await;
                    let mut state = state.write().await;
                    *state = CircuitState::HalfOpen;
                });

                Err(CircuitBreakerError::Inner(e))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    Open,

    #[error("Inner error: {0}")]
    Inner(E),
}
```

## Docker 部署

```dockerfile
# Dockerfile
FROM rust:1.83-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/agent-server /app/agent-server

EXPOSE 3000

CMD ["./agent-server"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  agent:
    build: .
    ports:
      - "3000:3000"
    environment:
      - APP_ENV=production
      - LLM_API_KEY=${OPENAI_API_KEY}
      - DATABASE_URL=postgresql://postgres:password@db:5432/agent
    depends_on:
      - db
      - redis

  db:
    image: postgres:16
    environment:
      - POSTGRES_DB=agent
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine

volumes:
  postgres_data:
```
