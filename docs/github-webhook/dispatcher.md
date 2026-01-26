# 事件分发

> [返回目录](README.md)

## 1. 事件分发器

```rust
// src/webhooks/github/dispatcher.rs

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::error::WebhookError;

/// Webhook 事件处理器 trait
#[async_trait]
pub trait WebhookHandler: Send + Sync {
    /// 返回处理的事件类型
    fn event_type(&self) -> &'static str;

    /// 处理事件
    async fn handle(&self, payload: &[u8]) -> Result<(), WebhookError>;
}

/// 事件分发器
pub struct WebhookDispatcher {
    handlers: HashMap<String, Arc<dyn WebhookHandler>>,
}

impl WebhookDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 注册事件处理器
    pub fn register<H: WebhookHandler + 'static>(&mut self, handler: H) -> &mut Self {
        let event_type = handler.event_type().to_string();
        self.handlers.insert(event_type, Arc::new(handler));
        self
    }

    /// 分发事件到对应处理器
    pub async fn dispatch(&self, event: &str, payload: &[u8]) -> Result<(), WebhookError> {
        match self.handlers.get(event) {
            Some(handler) => handler.handle(payload).await,
            None => {
                tracing::debug!(event = %event, "No handler registered, ignoring");
                Ok(())
            }
        }
    }
}

impl Default for WebhookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
```

## 2. 幂等性检查 (内存实现)

```rust
// src/webhooks/github/idempotency.rs

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 幂等性检查器 (内存实现)
///
/// 生产环境建议使用 Redis 替代
pub struct IdempotencyChecker {
    processed: Arc<RwLock<HashMap<String, Instant>>>,
    ttl: Duration,
}

impl IdempotencyChecker {
    pub fn new(ttl: Duration) -> Self {
        let checker = Self {
            processed: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        };

        // 启动清理任务
        let processed = checker.processed.clone();
        let ttl = checker.ttl;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let mut map = processed.write().await;
                let now = Instant::now();
                map.retain(|_, &mut v| now.duration_since(v) < ttl);
            }
        });

        checker
    }

    /// 检查并标记为已处理
    /// 返回 true 表示是新请求，false 表示重复请求
    pub async fn check_and_mark(&self, delivery_id: &str) -> bool {
        let mut map = self.processed.write().await;

        if map.contains_key(delivery_id) {
            return false;
        }

        map.insert(delivery_id.to_string(), Instant::now());
        true
    }
}
```

## 3. Redis 实现 (生产环境推荐)

```rust
// 生产环境推荐使用 Redis 实现

pub struct RedisIdempotencyChecker {
    redis: deadpool_redis::Pool,
    ttl: Duration,
}

impl RedisIdempotencyChecker {
    pub async fn check_and_mark(&self, delivery_id: &str) -> Result<bool, Error> {
        let mut conn = self.redis.get().await?;
        let key = format!("webhook:delivery:{}", delivery_id);

        // SET NX EX: 仅当 key 不存在时设置，并设置过期时间
        let result: Option<String> = redis::cmd("SET")
            .arg(&key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(self.ttl.as_secs())
            .query_async(&mut conn)
            .await?;

        Ok(result.is_some())
    }
}
```

## 4. 实现对比

| 特性 | 内存实现 | Redis 实现 |
|------|----------|------------|
| 部署复杂度 | 简单 | 需要 Redis |
| 多实例支持 | 不支持 | 支持 |
| 持久化 | 不支持 | 支持 |
| 内存占用 | 随事件增长 | 固定 |
| 适用场景 | 开发/单实例 | 生产环境 |
