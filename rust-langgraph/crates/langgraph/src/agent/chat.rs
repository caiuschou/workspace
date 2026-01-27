//! Chat Agent：单轮对话，调用 LLM 完成一问一答；支持流式与可选会话记忆。

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;

use crate::error::AgentError;
use crate::llm::{ChatMessage, ChatRequest, ChatStreamEvent, LlmClient, LlmStreamClient};
use crate::memory::Memory;
use crate::message::{Message, MessageRole as MemRole};
use crate::traits::{AsyncAgent, StreamAgent, StreamItem};

/// 从会话记忆消息转为 LLM 请求消息（Tool 在本 Sprint 中跳过）。
fn messages_to_chat(ms: &[Message]) -> Vec<ChatMessage> {
    ms.iter()
        .filter(|m| matches!(m.role, MemRole::System | MemRole::User | MemRole::Assistant))
        .map(|m| match m.role {
            MemRole::System => ChatMessage::system(&m.content),
            MemRole::User => ChatMessage::user(&m.content),
            MemRole::Assistant => ChatMessage::assistant(&m.content),
            MemRole::Tool => ChatMessage::user(&m.content), // 占位
        })
        .collect()
}

/// Chat Agent：持有一个 LLM 客户端、可选系统提示与可选会话记忆。
pub struct ChatAgent<C> {
    /// LLM 客户端。
    pub llm: C,
    /// 可选系统提示；若为 None，请求中不包含 system 消息。
    pub system_prompt: Option<String>,
    /// 可选会话记忆；有则多轮时带历史、并在每轮后写入用户与助手消息。
    pub memory: Option<Arc<dyn Memory>>,
}

impl<C: std::fmt::Debug> std::fmt::Debug for ChatAgent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatAgent")
            .field("llm", &self.llm)
            .field("system_prompt", &self.system_prompt)
            .field("memory", &self.memory.as_ref().map(|_| "..."))
            .finish()
    }
}

impl<C: LlmClient> ChatAgent<C> {
    /// 使用给定 LLM 客户端构造，无系统提示、无记忆。
    pub fn new(llm: C) -> Self {
        Self {
            llm,
            system_prompt: None,
            memory: None,
        }
    }

    /// 设置系统提示。
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// 设置会话记忆；多轮时自动带历史并在每轮后写入。
    pub fn with_memory(mut self, memory: Arc<dyn Memory>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// 构建请求消息列表：可选 system + 历史（来自 memory） + 当前用户输入。
    fn build_messages(&self, user_input: &str, history_limit: usize) -> Vec<ChatMessage> {
        let mut ms: Vec<ChatMessage> = Vec::new();
        if let Some(sys) = &self.system_prompt {
            ms.push(ChatMessage::system(sys.as_str()));
        }
        if let Some(m) = &self.memory {
            let hist = m.get(history_limit);
            ms.extend(messages_to_chat(&hist));
        }
        ms.push(ChatMessage::user(user_input));
        ms
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
        const HISTORY_LIMIT: usize = 64;
        let messages = self.build_messages(&input, HISTORY_LIMIT);
        let req = ChatRequest {
            messages,
            temperature: None,
            max_tokens: None,
        };
        let out = self
            .llm
            .chat(req)
            .await
            .map(|r| r.content)
            .map_err(|e| AgentError::ExecutionFailed(e.to_string()))?;
        if let Some(m) = &self.memory {
            m.add(Message::user(&input));
            m.add(Message::assistant(&out));
        }
        Ok(out)
    }
}

impl<C: LlmStreamClient> StreamAgent for ChatAgent<C> {
    type Input = String;
    type StreamItem = ChatStreamEvent;
    type Error = AgentError;

    fn name(&self) -> &str {
        "ChatAgent"
    }

    fn run_stream(
        &self,
        input: Self::Input,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<Self::StreamItem, Self::Error>> + Send + '_>> {
        const HISTORY_LIMIT: usize = 64;
        let messages = self.build_messages(&input, HISTORY_LIMIT);
        let req = ChatRequest {
            messages,
            temperature: None,
            max_tokens: None,
        };
        let memory = self.memory.clone();
        let user_input = input;
        let stream = self.llm.chat_stream(req);
        let mapped = stream.map(move |ev| {
            if let ChatStreamEvent::Done(content) = &ev
                && let Some(m) = &memory
            {
                m.add(Message::user(&user_input));
                m.add(Message::assistant(content));
            }
            match ev {
                ChatStreamEvent::Error(e) => Err(AgentError::ExecutionFailed(e.to_string())),
                other => Ok(other),
            }
        });
        Box::pin(mapped)
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
        assert_eq!(AsyncAgent::name(&agent), "ChatAgent");
    }
}
