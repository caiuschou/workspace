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
