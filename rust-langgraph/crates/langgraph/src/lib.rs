//! LangGraph-style minimal agents in Rust: state-in, state-out.
//!
//! Design: [docs/rust-langgraph/09-minimal-agent-design.md](https://github.com/.../09-minimal-agent-design.md).
//! One state type, one node per `Agent::run`, no separate Input/Output.
//! Concrete agents and state types (e.g. EchoAgent, AgentState) are implemented in examples, not in the framework.

pub mod error;
pub mod message;
pub mod traits;

pub use error::AgentError;
pub use message::Message;
pub use traits::Agent;
