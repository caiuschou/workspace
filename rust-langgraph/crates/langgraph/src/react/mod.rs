//! ReAct graph nodes: Think, Act, Observe.
//!
//! Design: [docs/rust-langgraph/13-react-agent-design.md](https://github.com/.../13-react-agent-design.md) §8.3 stage 3.
//! Three nodes implementing Node<ReActState> for the minimal ReAct chain
//! think → act → observe (linear, then conditional edge in stage 5).

mod act_node;
mod observe_node;
mod think_node;

pub use act_node::ActNode;
pub use observe_node::ObserveNode;
pub use think_node::ThinkNode;

/// Default system prompt for ReAct agents.
///
/// Prepend as the first message in `ReActState::messages` when building state
/// so the LLM follows think → act → observe behavior. Callers can use a custom
/// system message instead; ThinkNode does not inject this automatically.
pub const REACT_SYSTEM_PROMPT: &str = "You are a ReAct agent. Think step by step. When you need to use a tool, use it. After observing tool results, continue reasoning or respond to the user.";
