//! LangGraph-style minimal agents in Rust: state-in, state-out.
//!
//! Design: [docs/rust-langgraph/09-minimal-agent-design.md](https://github.com/.../09-minimal-agent-design.md).
//! One state type, one node per `Agent::run`, no separate Input/Output.
//! State graph: [docs/rust-langgraph/11-state-graph-design.md].
//! Concrete agents and state types (e.g. EchoAgent, AgentState) are implemented in examples, not in the framework.

pub mod error;
pub mod graph;
pub mod llm;
pub mod message;
pub mod react;
pub mod state;
pub mod tool_source;
pub mod traits;

pub use error::AgentError;
pub use graph::{CompilationError, CompiledStateGraph, Next, Node, StateGraph};
pub use llm::{LlmClient, LlmResponse, MockLlm};
#[cfg(feature = "openai")]
pub use llm::{ChatOpenAI, ChatZhipu};
pub use message::Message;
pub use state::{ReActState, ToolCall, ToolResult};
pub use react::{ActNode, ObserveNode, ThinkNode};
pub use tool_source::{MockToolSource, ToolCallContent, ToolSource, ToolSourceError, ToolSpec};
pub use traits::Agent;
