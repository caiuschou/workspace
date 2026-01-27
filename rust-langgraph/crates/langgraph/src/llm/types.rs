//! LLM 请求/响应与用量类型。

use serde::{Deserialize, Serialize};

/// 单条对话消息的角色。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// 系统提示。
    System,
    /// 用户消息。
    User,
    /// 助手回复。
    Assistant,
}

/// 单条对话消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 角色。
    pub role: MessageRole,
    /// 内容。
    pub content: String,
}

impl ChatMessage {
    /// 构造系统消息。
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    /// 构造用户消息。
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// 构造助手消息。
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// 聊天请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// 消息列表（按顺序）。
    pub messages: Vec<ChatMessage>,
    /// 可选：温度，未设置时由客户端默认值决定。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// 可选：最大生成 token 数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl ChatRequest {
    /// 从用户内容构造单轮请求（无 system）。
    pub fn single_turn(user_content: impl Into<String>) -> Self {
        Self {
            messages: vec![ChatMessage::user(user_content)],
            temperature: None,
            max_tokens: None,
        }
    }

    /// 从系统提示 + 用户内容构造单轮请求。
    pub fn with_system(
        system_prompt: impl Into<String>,
        user_content: impl Into<String>,
    ) -> Self {
        Self {
            messages: vec![
                ChatMessage::system(system_prompt),
                ChatMessage::user(user_content),
            ],
            temperature: None,
            max_tokens: None,
        }
    }
}

/// Token 用量。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// 提示消耗的 token 数。
    pub prompt_tokens: u32,
    /// 补全消耗的 token 数。
    pub completion_tokens: u32,
}

/// 聊天响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// 助手回复正文（单条时取 content；多条或流式时可后续扩展）。
    pub content: String,
    /// Token 用量。
    #[serde(default)]
    pub usage: Usage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_request_single_turn_roundtrip() {
        let req = ChatRequest::single_turn("你好");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, MessageRole::User);
        assert_eq!(req.messages[0].content, "你好");
    }

    #[test]
    fn chat_request_with_system_roundtrip() {
        let req = ChatRequest::with_system("你是一个助手", "你好");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, MessageRole::System);
        assert_eq!(req.messages[1].role, MessageRole::User);
    }

    #[test]
    fn usage_default() {
        let u = Usage::default();
        assert_eq!(u.prompt_tokens, 0);
        assert_eq!(u.completion_tokens, 0);
    }

    #[test]
    fn chat_message_constructors() {
        assert_eq!(ChatMessage::system("s").role, MessageRole::System);
        assert_eq!(ChatMessage::user("u").role, MessageRole::User);
        assert_eq!(ChatMessage::assistant("a").role, MessageRole::Assistant);
    }
}
