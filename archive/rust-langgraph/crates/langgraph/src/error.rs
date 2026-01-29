//! Agent 与执行相关错误类型。
//!
//! - `AgentError`: 通用 Agent 执行错误（S1）
//! - `StateError`: 状态机转换错误（S4）
//! - `ToolError`: 工具执行与校验错误（S4）
//! - `ActorError`: Actor/Worker 通信与处理错误（S6）

use thiserror::Error;

/// Agent 执行过程中的最小错误枚举。
#[derive(Debug, Error)]
pub enum AgentError {
    /// 执行失败，附带原因描述。
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

/// 状态机转换或运行时的错误。
///
/// 与 `StateMachine::transition` 及 `Runner::run` 配合使用。
#[derive(Debug, Error)]
pub enum StateError {
    /// 无效状态转换。
    #[error("invalid transition: {0}")]
    InvalidTransition(String),
    /// 超过最大步数限制。
    #[error("max steps exceeded: {0}")]
    MaxStepsExceeded(usize),
    /// 内部错误（如 LLM、工具调用失败）。
    #[error("internal: {0}")]
    Internal(String),
}

/// 参数校验错误，用于 `validate_args` 与 `ToolRegistry::execute`。
///
/// Produced by `validate_args(schema, args)` when required fields are missing or types mismatch.
/// Converts to `ToolError::ValidationFailed` when used in `ToolRegistry::execute`.
#[derive(Debug, Error, Clone)]
#[error("validation failed: {0}")]
pub struct ValidationError(pub String);

/// 工具执行与参数校验错误。
///
/// 与 `Tool::execute` 及 `ToolRegistry::execute` 配合使用。
#[derive(Debug, Error)]
pub enum ToolError {
    /// 工具不存在。
    #[error("tool not found: {0}")]
    NotFound(String),
    /// 参数校验失败（如缺少必填字段、类型不符）。
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// 执行失败。
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

impl From<ValidationError> for ToolError {
    fn from(e: ValidationError) -> Self {
        ToolError::ValidationFailed(e.0)
    }
}

/// Actor 与 Worker 通信、处理、超时等错误。
///
/// 与 `Handler::handle`、`ActorRef::request`、`Supervisor` 等配合使用。
#[derive(Debug, Error)]
pub enum ActorError {
    /// 处理消息时失败（如 Worker 执行异常）。
    #[error("handle failed: {0}")]
    HandleFailed(String),
    /// 发送超时（如 `send_timeout` 或 `request` 在限定时间内未完成）。
    #[error("send timeout")]
    SendTimeout,
    /// 信道已关闭，无法发送。
    #[error("channel closed")]
    ChannelClosed,
    /// 请求-响应时对方未返回或返回错误。
    #[error("request failed: {0}")]
    RequestFailed(String),
}
