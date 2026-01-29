//! Mock LLM for tests and examples.
//!
//! Design: [docs/rust-langgraph/13-react-agent-design.md] §8.2 stage 2.2.
//! Returns fixed assistant message and optional fixed ToolCall (e.g. get_time);
//! configurable "no tool_calls" to test END path.

use async_trait::async_trait;

use crate::error::AgentError;
use crate::llm::{LlmClient, LlmResponse};
use crate::message::Message;
use crate::state::ToolCall;

/// Mock LLM: fixed assistant text and optional tool_calls.
///
/// Configurable to return one fixed ToolCall (e.g. get_time) or no tool_calls,
/// so the graph can run one round (think → act → observe → END) or test END
/// after think. Used by ThinkNode in tests and ReAct linear example.
///
/// **Interaction**: Implements `LlmClient`; used by ThinkNode.
pub struct MockLlm {
    /// Assistant message content to return.
    content: String,
    /// Tool calls to return; empty = no tools, observe → END.
    tool_calls: Vec<ToolCall>,
}

impl MockLlm {
    /// Build a mock that returns one assistant message and one tool call (get_time).
    ///
    /// Aligns with 13-react-agent-design §8.2: "固定返回一条 assistant 消息 + 固定一条 ToolCall（如 get_time）".
    pub fn with_get_time_call() -> Self {
        Self {
            content: "I'll check the time.".to_string(),
            tool_calls: vec![ToolCall {
                name: "get_time".to_string(),
                arguments: "{}".to_string(),
                id: Some("call-1".to_string()),
            }],
        }
    }

    /// Build a mock that returns assistant text and no tool_calls (END path).
    pub fn with_no_tool_calls(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            tool_calls: vec![],
        }
    }

    /// Build with custom content and tool_calls.
    pub fn new(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: content.into(),
            tool_calls,
        }
    }

    /// Set content (builder).
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Set tool_calls (builder).
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = tool_calls;
        self
    }
}

#[async_trait]
impl LlmClient for MockLlm {
    async fn complete(&self, _messages: &[Message]) -> Result<LlmResponse, AgentError> {
        Ok(LlmResponse {
            content: self.content.clone(),
            tool_calls: self.tool_calls.clone(),
        })
    }
}
