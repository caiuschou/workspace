//! Compiled state graph: immutable, supports invoke only.
//!
//! Built by `StateGraph::compile`. Holds nodes and linear edge order.
//! Used by callers to run the graph via `invoke(state)`.
//! Supports conditional edges: node returns `Next` to continue, jump, or end. See 13-react-agent-design §8.5.

use std::collections::HashMap;

use crate::error::AgentError;

use super::Next;
use super::Node;

/// Compiled graph: immutable structure, supports invoke only.
///
/// Created by `StateGraph::compile()`. Runs from first node; uses each node's
/// returned `Next` to choose next node (Continue = linear order, Node(id) = jump, End = stop).
/// **Interaction**: Built from `StateGraph`; callers use `invoke(state)` to execute.
pub struct CompiledStateGraph<S> {
    pub(super) nodes: HashMap<String, Box<dyn Node<S>>>,
    pub(super) edge_order: Vec<String>,
}

impl<S> CompiledStateGraph<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Runs the graph with the given state. Starts at the first node in edge order;
    /// after each node, uses returned `Next` to continue linear order, jump to a node, or end.
    ///
    /// - `Next::Continue`: run the next node in edge_order, or end if last.
    /// - `Next::Node(id)`: run the node with that id next.
    /// - `Next::End`: stop and return current state.
    pub async fn invoke(&self, state: S) -> Result<S, AgentError> {
        let mut state = state;
        let mut current_id = self
            .edge_order
            .first()
            .cloned()
            .ok_or_else(|| AgentError::ExecutionFailed("empty graph".into()))?;

        loop {
            let node = self
                .nodes
                .get(&current_id)
                .expect("compiled graph has all nodes");
            let (new_state, next) = node.run(state).await?;
            state = new_state;

            match next {
                Next::End => return Ok(state),
                Next::Node(id) => current_id = id,
                Next::Continue => {
                    let pos = self
                        .edge_order
                        .iter()
                        .position(|x| x == &current_id)
                        .expect("current node in edge_order");
                    let next_pos = pos + 1;
                    if next_pos >= self.edge_order.len() {
                        return Ok(state);
                    }
                    current_id = self.edge_order[next_pos].clone();
                }
            }
        }
    }
}
