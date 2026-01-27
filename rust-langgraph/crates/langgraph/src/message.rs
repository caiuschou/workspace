//! 消息类型。
//!
//! 本 Sprint 仅需 `UserMessage` 作为 Echo 输入，后续可扩展为 User/Assistant/System/Tool。

/// 用户消息，用于本 Sprint 的 Echo 输入。
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
