//! Mock LLM 客户端，用于测试与示例（无真实 API 调用）。

use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use futures::Stream;

use super::client::LlmClient;
use super::error::LlmError;
use super::stream::{ChatStreamEvent, LlmStreamClient};
use super::types::{ChatRequest, ChatResponse, Usage};

/// 按调用顺序返回预设回复的 Mock，用于 ReAct 等多轮调用示例。
#[derive(Debug)]
pub struct SequenceMockLlmClient {
    responses: Vec<String>,
    index: AtomicUsize,
}

impl SequenceMockLlmClient {
    /// 新建：每次 `chat` 返回 `responses[i % responses.len()]`。
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: AtomicUsize::new(0),
        }
    }

    /// 从字符串切片构造。
    pub fn from_slice(responses: &[&str]) -> Self {
        Self::new(responses.iter().map(|s| (*s).to_string()).collect())
    }
}

#[async_trait]
impl LlmClient for SequenceMockLlmClient {
    async fn chat(&self, _req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let i = self.index.fetch_add(1, Ordering::Relaxed);
        let content = self
            .responses
            .get(i % self.responses.len())
            .cloned()
            .unwrap_or_default();
        Ok(ChatResponse {
            content,
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
            },
        })
    }
}

/// Mock 客户端：按配置返回固定内容或简单回显最后一条用户消息。
#[derive(Debug)]
pub struct MockLlmClient {
    /// 若为 Some，则始终返回该字符串；否则回显最后一条用户消息内容。
    fixed_response: Option<String>,
}

impl MockLlmClient {
    /// 构造始终返回固定句的 Mock。
    pub fn with_fixed_response(response: impl Into<String>) -> Self {
        Self {
            fixed_response: Some(response.into()),
        }
    }

    /// 构造回显最后一条用户消息的 Mock。
    pub fn echo() -> Self {
        Self {
            fixed_response: None,
        }
    }

    /// 默认：回显。
    pub fn new() -> Self {
        Self::echo()
    }
}

impl Default for MockLlmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let content = if let Some(ref s) = self.fixed_response {
            s.clone()
        } else {
            req.messages
                .iter()
                .rev()
                .find(|m| matches!(m.role, super::types::MessageRole::User))
                .map(|m| m.content.clone())
                .unwrap_or_else(|| "（无用户输入）".to_string())
        };
        let prompt_tokens = req
            .messages
            .iter()
            .map(|m| m.content.len() / 2)
            .sum::<usize>()
            .min(u32::MAX as usize) as u32;
        let completion_tokens = (content.len() / 2).min(u32::MAX as usize) as u32;
        Ok(ChatResponse {
            content,
            usage: Usage {
                prompt_tokens,
                completion_tokens,
            },
        })
    }
}

impl LlmStreamClient for MockLlmClient {
    fn chat_stream(
        &self,
        req: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = ChatStreamEvent> + Send + '_>> {
        let content = if let Some(ref s) = self.fixed_response {
            s.clone()
        } else {
            req.messages
                .iter()
                .rev()
                .find(|m| matches!(m.role, super::types::MessageRole::User))
                .map(|m| m.content.clone())
                .unwrap_or_else(|| "（无用户输入）".to_string())
        };
        let full = content.clone();
        let tokens: Vec<ChatStreamEvent> = content
            .chars()
            .map(|c| ChatStreamEvent::Token(c.to_string()))
            .chain(std::iter::once(ChatStreamEvent::Done(full)))
            .collect();
        Box::pin(futures::stream::iter(tokens))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::types::ChatRequest;

    #[tokio::test]
    async fn mock_echo_returns_last_user_message() {
        let client = MockLlmClient::echo();
        let req = ChatRequest::single_turn("你好");
        let res = client.chat(req).await.unwrap();
        assert_eq!(res.content, "你好");
    }

    #[tokio::test]
    async fn mock_fixed_returns_given_response() {
        let client = MockLlmClient::with_fixed_response("这是固定回复");
        let req = ChatRequest::single_turn("任意问题");
        let res = client.chat(req).await.unwrap();
        assert_eq!(res.content, "这是固定回复");
    }
}
