//! LLM 客户端与请求/响应类型。
//!
//! - `LlmClient`：异步对话接口
//! - `ChatRequest` / `ChatResponse` / `Usage`：请求、响应与用量
//! - `LlmError`：调用错误枚举

mod client;
mod error;
mod mock;
#[cfg(feature = "openai")]
mod openai;
mod stream;
mod types;

pub use client::LlmClient;
pub use error::LlmError;
pub use mock::MockLlmClient;
#[cfg(feature = "openai")]
pub use openai::{OpenAiClient, OpenAiConfig};
pub use stream::{ChatStreamEvent, LlmStreamClient};
pub use types::{ChatMessage, ChatRequest, ChatResponse, MessageRole, Usage};
