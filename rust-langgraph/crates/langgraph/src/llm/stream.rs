//! 流式 Chat 事件与流式客户端扩展。
//!
//! - `ChatStreamEvent`: 流式事件枚举
//! - `LlmStreamClient`: 支持 chat_stream 的 LLM 客户端

use std::pin::Pin;

use futures::Stream;

use super::error::LlmError;
use super::types::ChatRequest;

/// 流式 Chat 事件。
#[derive(Debug, Clone)]
pub enum ChatStreamEvent {
    /// 单个 token 或文本片段。
    Token(String),
    /// 流结束，携带完整内容。
    Done(String),
    /// 流中错误（如解析失败、连接断开）。
    Error(LlmError),
}

/// 支持流式调用的 LLM 客户端。
///
/// 在 `LlmClient` 基础上增加 `chat_stream`，返回按 chunk 产生的事件流。
pub trait LlmStreamClient: super::client::LlmClient + Send + Sync {
    /// 发起流式对话，返回事件流。
    ///
    /// 事件顺序一般为若干 `Token`、一个 `Done`；出错时产生 `Error` 并结束。
    fn chat_stream(
        &self,
        req: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = ChatStreamEvent> + Send + '_>>;
}
