//! 核心 trait 定义。
//!
//! 本 Sprint 仅需最小 `Agent` trait。

/// 最小 Agent trait。
///
/// - `name`: 标识名称
/// - `run(Input) -> Result<Output, Error>`: 同步执行，本 Sprint 用于 Echo
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
