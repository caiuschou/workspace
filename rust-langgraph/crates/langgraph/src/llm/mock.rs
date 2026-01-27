//! Mock LLM 客户端，用于测试与示例（无真实 API 调用）。

use async_trait::async_trait;

use super::client::LlmClient;
use super::error::LlmError;
use super::types::{ChatRequest, ChatResponse, Usage};

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
