//! Agent 实现。
//!
//! - `EchoAgent`: 同步回显（S1）
//! - `ChatAgent`: 单轮 LLM 对话（S2）
//! - ReAct agent: 见独立包 `langgraph-react`

mod chat;
mod echo;

pub use chat::ChatAgent;
pub use echo::EchoAgent;

