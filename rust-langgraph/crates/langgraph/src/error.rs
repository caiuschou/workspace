//! Agent 与执行相关错误类型。
//!
//! 本 Sprint 仅包含最小 `AgentError`，后续可扩展为 LlmError、ToolError 等。

use thiserror::Error;

/// Agent 执行过程中的最小错误枚举。
#[derive(Debug, Error)]
pub enum AgentError {
    /// 执行失败，附带原因描述。
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}
