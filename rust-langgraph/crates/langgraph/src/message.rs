//! 消息类型。
//!
//! - `UserMessage`: Echo 等场景的简单用户输入（S1）
//! - `Message`: 会话记忆用消息枚举，含 User/Assistant/System/Tool（S3）

use std::time::SystemTime;

/// 用户消息，用于 Echo 等场景的简单输入。
#[derive(Debug, Clone)]
pub struct UserMessage {
    /// 消息内容。
    pub content: String,
}

impl UserMessage {
    /// 从内容构造用户消息。
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

/// 会话记忆用消息：带角色、内容与可选时间戳。
#[derive(Debug, Clone)]
pub struct Message {
    /// 角色。
    pub role: MessageRole,
    /// 正文。
    pub content: String,
    /// 可选时间戳（写入时若为 None 可自动填充）。
    pub timestamp: Option<SystemTime>,
}

/// 消息角色（与 LLM 的 ChatMessage 对齐，并支持 Tool）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    /// 系统提示。
    System,
    /// 用户消息。
    User,
    /// 助手回复。
    Assistant,
    /// 工具调用或工具结果。
    Tool,
}

impl Message {
    /// 构造系统消息。
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            timestamp: None,
        }
    }

    /// 构造用户消息。
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            timestamp: None,
        }
    }

    /// 构造助手消息。
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp: None,
        }
    }

    /// 构造工具消息。
    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            timestamp: None,
        }
    }

    /// 设置时间戳。
    pub fn with_timestamp(mut self, t: SystemTime) -> Self {
        self.timestamp = Some(t);
        self
    }
}

/// 工具调用描述（最小结构，S3 仅占位，S4 扩展）。
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// 工具名。
    pub name: String,
    /// 参数字符串（如 JSON）。
    pub arguments: String,
}

/// 工具调用结果（最小结构）。
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// 工具名。
    pub name: String,
    /// 返回内容。
    pub content: String,
}
