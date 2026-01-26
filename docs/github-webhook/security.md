# 安全验证

> [返回目录](README.md)

## 1. 签名验证

```rust
// src/webhooks/github/verify.rs

use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// 验证 GitHub Webhook 签名
///
/// # Security
/// 使用常量时间比较防止时序攻击
pub fn verify_signature(secret: &SecretString, payload: &[u8], signature: &str) -> bool {
    // 签名格式: sha256=<hex_digest>
    let signature = match signature.strip_prefix("sha256=") {
        Some(sig) => sig,
        None => return false,
    };

    let signature_bytes = match hex::decode(signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.expose_secret().as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    mac.update(payload);

    // verify_slice 使用常量时间比较
    mac.verify_slice(&signature_bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature_valid() {
        let secret = SecretString::new("test_secret".into());
        let payload = b"test payload";

        let mut mac = HmacSha256::new_from_slice(b"test_secret").unwrap();
        mac.update(payload);
        let result = mac.finalize();
        let signature = format!("sha256={}", hex::encode(result.into_bytes()));

        assert!(verify_signature(&secret, payload, &signature));
    }

    #[test]
    fn test_verify_signature_invalid() {
        let secret = SecretString::new("test_secret".into());
        let payload = b"test payload";

        assert!(!verify_signature(&secret, payload, "sha256=invalid"));
        assert!(!verify_signature(&secret, payload, "invalid_format"));
    }
}
```

## 2. Webhook 错误类型

```rust
// src/webhooks/github/error.rs

use crate::error::AppError;

/// Webhook 特定错误
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Missing signature")]
    MissingSignature,

    #[error("Missing required header: {0}")]
    MissingHeader(&'static str),

    #[error("Failed to parse payload: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Duplicate delivery: {0}")]
    DuplicateDelivery(String),
}

impl From<WebhookError> for AppError {
    fn from(err: WebhookError) -> Self {
        match err {
            WebhookError::InvalidSignature | WebhookError::MissingSignature => {
                AppError::Unauthorized(err.to_string())
            }
            WebhookError::MissingHeader(_) | WebhookError::ParseError(_) => {
                AppError::BadRequest(err.to_string())
            }
            WebhookError::DuplicateDelivery(_) => {
                // 幂等处理，不算错误
                AppError::Conflict(err.to_string())
            }
        }
    }
}
```

## 3. 错误映射说明

| 错误类型 | HTTP 状态码 | 说明 |
|----------|-------------|------|
| `InvalidSignature` | 401 Unauthorized | 签名验证失败 |
| `MissingSignature` | 401 Unauthorized | 缺少签名头 |
| `MissingHeader` | 400 Bad Request | 缺少必要的请求头 |
| `ParseError` | 400 Bad Request | JSON 解析失败 |
| `DuplicateDelivery` | 409 Conflict | 重复的投递 ID（幂等处理） |
