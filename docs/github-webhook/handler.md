# 请求处理

> [返回目录](README.md)

## 1. 状态结构

```rust
// src/webhooks/github/handler.rs

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use secrecy::SecretString;

use super::{
    dispatcher::WebhookDispatcher,
    idempotency::IdempotencyChecker,
    models::GitHubWebhookHeaders,
    verify::verify_signature,
};

#[derive(Clone)]
pub struct WebhookState {
    pub webhook_secret: SecretString,
    pub dispatcher: WebhookDispatcher,
    pub idempotency: IdempotencyChecker,
}
```

## 2. 请求头提取

```rust
/// 从 HeaderMap 提取 GitHub Webhook 请求头
fn extract_webhook_headers(headers: &HeaderMap) -> Result<GitHubWebhookHeaders, WebhookError> {
    let event = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .ok_or(WebhookError::MissingHeader("x-github-event"))?;

    let delivery_id = headers
        .get("x-github-delivery")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .ok_or(WebhookError::MissingHeader("x-github-delivery"))?;

    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    Ok(GitHubWebhookHeaders {
        event,
        delivery_id,
        signature,
    })
}
```

## 3. 主处理器

```rust
pub async fn github_webhook_handler(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Result<Response, WebhookError> {
    let hdrs = extract_webhook_headers(&headers)?;

    // 1. 验证签名
    if let Some(sig) = &hdrs.signature {
        if !verify_signature(&state.webhook_secret, &body, sig) {
            tracing::warn!(
                delivery_id = %hdrs.delivery_id,
                "Invalid webhook signature"
            );
            return Err(WebhookError::InvalidSignature);
        }
    } else {
        return Err(WebhookError::MissingSignature);
    }

    // 2. 幂等性检查
    if !state.idempotency.check_and_mark(&hdrs.delivery_id).await {
        tracing::info!(
            delivery_id = %hdrs.delivery_id,
            "Duplicate delivery, ignoring"
        );
        return Ok(StatusCode::ACCEPTED.into_response());
    }

    // 3. 分发事件
    state
        .dispatcher
        .dispatch(&hdrs.event, &body)
        .await
        .map_err(|e| {
            tracing::error!(
                delivery_id = %hdrs.delivery_id,
                event = %hdrs.event,
                error = %e,
                "Failed to dispatch webhook"
            );
            e
        })?;

    tracing::info!(
        delivery_id = %hdrs.delivery_id,
        event = %hdrs.event,
        "Webhook processed"
    );

    // 4. 快速返回 202
    Ok(StatusCode::ACCEPTED.into_response())
}
```

## 4. 健康检查

```rust
#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
}

pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}
```

## 处理流程图

```
┌─────────────────────────────────────────────────────────┐
│              GitHub Webhook 请求处理流程                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │   提取头     │──>│  签名验证    │──>│  幂等性检查   │ │
│  │  Headers    │    │  Signature  │    │ Idempotency │ │
│  └─────────────┘    └─────────────┘    └──────┬──────┘ │
│                                              │          │
│  ┌─────────────┐    ┌─────────────┐         │          │
│  │ 202 Accepted│<──│  事件分发    │<────────┘          │
│  │   返回      │    │  Dispatch   │                     │
│  └─────────────┘    └──────┬──────┘                     │
│                             │                            │
│  ┌──────────────────────────▼─────────┐                │
│  │           异步处理队列               │                │
│  │  (tokio::spawn / background task)   │                │
│  └─────────────────────────────────────┘                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```
