# 测试指南

> [返回目录](README.md)

## 1. 安全检查清单

| 检查项 | 说明 | 验证方式 |
|--------|------|----------|
| 签名验证 | 验证 `X-Hub-Signature-256` | 发送错误签名，确保返回 401 |
| 幂等性 | 基于 `X-GitHub-Delivery` 去重 | 重放相同请求，确保只处理一次 |
| 超时处理 | 在 10 秒内返回响应 | 模拟长时间处理，确保 202 响应 |
| 输入验证 | 验证 payload 格式 | 发送无效 JSON，确保返回 400 |
| 速率限制 | 防止滥用 | 使用 `tower_governor` |

## 2. 本地测试

使用 `curl` 测试 Webhook 端点：

```bash
# 设置环境变量
export GITHUB_WEBHOOK_SECRET="test_secret"

# 生成签名
PAYLOAD='{"zen":"Keep it logically awesome.","hook_id":123456}'
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "test_secret" | awk '{print "sha256="$2}')

# 发送 ping 事件
curl -X POST http://localhost:3000/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: ping" \
  -H "X-GitHub-Delivery: test-delivery-123" \
  -H "X-Hub-Signature-256: $SIGNATURE" \
  -d "$PAYLOAD"

# 健康检查
curl http://localhost:3000/webhooks/github/health
```

## 3. 集成测试

```rust
// tests/webhook_test.rs

use axum::{
    body::Body,
    http::{header, HeaderMap, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_ping_event() {
    let app = create_test_app().await;

    let payload = r#"{"zen":"Keep it logically awesome."}"#;

    let signature = generate_signature("test_secret", payload.as_bytes());

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-github-event", "ping")
                .header("x-github-delivery", "test-123")
                .header("x-hub-signature-256", signature)
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_invalid_signature() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-github-event", "ping")
                .header("x-github-delivery", "test-123")
                .header("x-hub-signature-256", "sha256=invalid")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_idempotency() {
    let app = create_test_app().await;

    let payload = r#"{"zen":"Keep it logically awesome."}"#;
    let signature = generate_signature("test_secret", payload.as_bytes());

    let request = || {
        axum::http::Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-github-event", "ping")
            .header("x-github-delivery", "duplicate-test")
            .header("x-hub-signature-256", signature.clone())
            .body(Body::from(payload))
            .unwrap()
    };

    // 第一次请求
    let response1 = app.clone().oneshot(request()).await.unwrap();
    assert_eq!(response1.status(), StatusCode::ACCEPTED);

    // 重复请求
    let response2 = app.clone().oneshot(request()).await.unwrap();
    assert_eq!(response2.status(), StatusCode::ACCEPTED);

    // 验证只处理了一次
    // ... 根据具体实现验证
}

fn generate_signature(secret: &str, payload: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload);
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}
```

## 4. 后续扩展

### 4.1 支持更多事件

```rust
// 添加新事件处理器
impl WebhookHandler for WorkflowRunHandler {
    fn event_type(&self) -> &'static str {
        "workflow_run"
    }

    async fn handle(&self, payload: &[u8]) -> Result<(), WebhookError> {
        let event: WorkflowRunEvent = serde_json::from_slice(payload)?;
        // 处理 CI/CD 状态更新
        Ok(())
    }
}
```

### 4.2 重试机制

对于暂时性错误，实现指数退避重试：

```rust
use tokio::time::{sleep, Duration};

async fn handle_with_retry<F, Fut, T, E>(mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut retries = 3;
    let mut delay = Duration::from_millis(100);

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries > 0 => {
                retries -= 1;
                sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 4.3 Webhook 日志存储

持久化存储 Webhook 事件以便调试：

```rust
#[derive(serde::Serialize)]
struct WebhookLog {
    delivery_id: String,
    event: String,
    received_at: DateTime<Utc>,
    payload: serde_json::Value,
    status: String,
    error: Option<String>,
}

async fn log_webhook_event(
    pool: &PgPool,
    log: WebhookLog,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO webhook_logs (delivery_id, event, received_at, payload, status, error)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        log.delivery_id,
        log.event,
        log.received_at,
        log.payload,
        log.status,
        log.error,
    )
    .execute(pool)
    .await?;

    Ok(())
}
```
