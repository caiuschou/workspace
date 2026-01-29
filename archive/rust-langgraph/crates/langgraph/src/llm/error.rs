//! LLM 调用相关错误类型。

use thiserror::Error;

/// LLM 调用过程中的错误枚举。
#[derive(Debug, Clone, Error)]
pub enum LlmError {
    /// API 返回错误（如 4xx/5xx 或业务错误）。
    #[error("api error: {0}")]
    ApiError(String),

    /// 限流（如 429）。
    #[error("rate limit: {0}")]
    RateLimit(String),

    /// 认证失败（如 401/403）。
    #[error("auth failed: {0}")]
    Auth(String),

    /// 请求参数无效。
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// 网络或连接错误。
    #[error("network error: {0}")]
    Network(String),

    /// 响应解析失败。
    #[error("parsing failed: {0}")]
    Parsing(String),

    /// 流已关闭。
    #[error("stream closed: {0}")]
    StreamClosed(String),
}
