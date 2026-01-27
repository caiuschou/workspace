//! 核心 trait 定义。
//!
//! - `Agent`: 同步执行，用于 Echo 等本地逻辑
//! - `AsyncAgent`: 异步执行，用于 Chat 等需调用 LLM 的场景

use async_trait::async_trait;

/// 最小 Agent trait（同步）。
///
/// - `name`: 标识名称
/// - `run(Input) -> Result<Output, Error>`: 同步执行，用于 Echo 等
pub trait Agent {
    /// 输入类型。
    type Input;
    /// 输出类型。
    type Output;
    /// 错误类型，实现时可使用 `crate::AgentError`。
    type Error: std::error::Error + Send + Sync + 'static;

    /// Agent 名称。
    fn name(&self) -> &str;

    /// 执行：给定输入，返回输出或错误。
    fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

/// 异步 Agent trait。
///
/// 用于需要异步 I/O（如 LLM 调用）的 Agent，如 ChatAgent。
#[async_trait]
pub trait AsyncAgent {
    /// 输入类型。
    type Input;
    /// 输出类型。
    type Output;
    /// 错误类型。
    type Error: std::error::Error + Send + Sync + 'static;

    /// Agent 名称。
    fn name(&self) -> &str;

    /// 异步执行：给定输入，返回输出或错误。
    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
