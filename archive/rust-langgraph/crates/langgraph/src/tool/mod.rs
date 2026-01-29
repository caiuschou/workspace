//! 工具 trait 与注册。
//!
//! - `Tool`: 工具接口（name、description、parameters_schema、execute）
//! - `ToolRegistry`: 注册与按名执行
//! - `validate_args`: 按 schema 的 required 校验 args（见 `tool::validation`）
//! - `ToolError`: 见 `crate::error::ToolError`

mod builtin;
mod compose;
mod file_ops;
mod http_request;
mod registry;
mod validation;

pub use builtin::CalculatorTool;
pub use compose::ToolChain;
pub use file_ops::FileOpsTool;
pub use http_request::{HttpFetcher, HttpRequestTool, MockHttpFetcher};
pub use registry::ToolRegistry;
pub use validation::validate_args;

#[cfg(feature = "http")]
pub use http_request::ReqwestHttpFetcher;

use crate::error::ToolError;
use serde_json::Value;

/// 工具 trait。
///
/// 与 `ToolRegistry` 配合：`register(Box<dyn Tool>)`、`execute(name, args)`。
/// `parameters_schema` 为最小 JSON Schema（如 `{"type":"object","properties":{...},"required":[...]}`），
/// 供 ReAct prompt 与参数校验使用。
pub trait Tool: Send + Sync {
    /// 工具名称，用于注册与调用。
    fn name(&self) -> &str;

    /// 工具描述，供 LLM 选择与生成调用参数。
    fn description(&self) -> &str;

    /// 参数 JSON Schema（最小：type、properties、required）。
    fn parameters_schema(&self) -> Value;

    /// 执行：传入已解析的参数字典，返回结果值或错误。
    fn execute(&self, args: Value) -> Result<Value, ToolError>;
}
