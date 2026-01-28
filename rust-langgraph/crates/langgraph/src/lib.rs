//! LangGraph 风格的类型安全 Agent 与状态机。
//!
//! 开发计划见仓库内 `docs/rust-langgraph/ROADMAP.md`。

pub mod actor;
pub mod agent;
pub mod error;
pub mod llm;
pub mod memory;
pub mod message;
pub mod state;
pub mod tool;
pub mod traits;

pub use agent::{ChatAgent, EchoAgent};
pub use error::{AgentError, StateError, ToolError, ValidationError};
pub use state::{Runner, StateMachine, StateTransition, DEFAULT_MAX_STEPS};
pub use llm::{ChatRequest, ChatResponse, ChatStreamEvent, LlmClient, LlmStreamClient, MockLlmClient, SequenceMockLlmClient, Usage};
#[cfg(feature = "openai")]
pub use llm::{OpenAiClient, OpenAiConfig};
pub use message::{Message, MessageRole, ToolCall, ToolResult, UserMessage};
pub use memory::{
    Embedder, EmbedderError, Memory, MemoryEmbedding, MemoryResult, MockEmbedder, SemanticMemory,
    SessionMemory, VectorMemory,
};
pub use tool::{
    CalculatorTool, FileOpsTool, HttpFetcher, HttpRequestTool, MockHttpFetcher, Tool, ToolChain,
    ToolRegistry, validate_args,
};
#[cfg(feature = "http")]
pub use tool::ReqwestHttpFetcher;
pub use traits::{Agent, AsyncAgent, StreamAgent, StreamItem};
