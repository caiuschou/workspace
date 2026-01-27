//! Chat Agent：单轮对话，调用 LLM 完成一问一答。

use async_trait::async_trait;

use crate::error::AgentError;
use crate::llm::{ChatRequest, LlmClient};
use crate::traits::AsyncAgent;

/// Chat Agent：持有一个 LLM 客户端与可选系统提示，单轮将用户输入转为请求、调用 LLM、返回回复。
#[derive(Debug)]
pub struct ChatAgent<C> {
    /// LLM 客户端。
    pub llm: C,
    /// 可选系统提示；若为 None，请求中不包含 system 消息。
    pub system_prompt: Option<String>,
}

impl<C: LlmClient> ChatAgent<C> {
    /// 使用给定 LLM 客户端构造，无系统提示。
    pub fn new(llm: C) -> Self {
        Self {
            llm,
            system_prompt: None,
        }
    }

    /// 设置系统提示。
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
}

#[async_trait]
impl<C: LlmClient + Send + Sync> AsyncAgent for ChatAgent<C> {
    type Input = String;
    type Output = String;
    type Error = AgentError;

    fn name(&self) -> &str {
        "ChatAgent"
    }

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let req = match &self.system_prompt {
            Some(sys) => ChatRequest::with_system(sys.as_str(), input),
            None => ChatRequest::single_turn(input),
        };
        self.llm
            .chat(req)
            .await
            .map(|r| r.content)
            .map_err(|e| AgentError::ExecutionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockLlmClient;

    #[tokio::test]
    async fn chat_agent_returns_llm_content() {
        let llm = MockLlmClient::with_fixed_response("固定回复");
        let agent = ChatAgent::new(llm);
        let out = agent.run("问什么都可以".to_string()).await.unwrap();
        assert_eq!(out, "固定回复");
    }

    #[tokio::test]
    async fn chat_agent_echo_without_system() {
        let llm = MockLlmClient::echo();
        let agent = ChatAgent::new(llm);
        let out = agent.run("你好".to_string()).await.unwrap();
        assert_eq!(out, "你好");
    }

    #[tokio::test]
    async fn chat_agent_name() {
        let llm = MockLlmClient::echo();
        let agent = ChatAgent::new(llm);
        assert_eq!(agent.name(), "ChatAgent");
    }
}
