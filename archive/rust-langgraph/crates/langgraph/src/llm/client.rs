//! LLM 客户端 trait 定义。

use async_trait::async_trait;

use crate::llm::error::LlmError;
use crate::llm::types::{ChatRequest, ChatResponse};

/// LLM 客户端：同步或异步发起对话并返回完整回复。
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 发起一次对话，返回完整内容与用量。
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError>;
}
