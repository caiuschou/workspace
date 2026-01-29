//! Compiled state graph: immutable, supports invoke only.
//!
//! Built by `StateGraph::compile`. Holds nodes and linear edge order.
//! Used by callers to run the graph via `invoke(state)`.

use std::collections::HashMap;

use crate::error::AgentError;
use crate::graph::node::Node;

/// Compiled graph: immutable structure, supports invoke only.
///
/// Created by `StateGraph::compile()`. Runs the linear chain START → n1 → n2 → … → END.
/// **Interaction**: Built from `StateGraph`; callers use `invoke(state)` to execute.
pub struct CompiledStateGraph<S> {
    pub(super) nodes: HashMap<String, Box<dyn Node<S>>>,
    pub(super) edge_order: Vec<String>,
}

impl<S> CompiledStateGraph<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Runs START → n1 → n2 → … → END with the given state. Returns final state.
    ///
    /// Each node is run in order; the output state of one node is the input
    /// state of the next. Errors from any node abort the run.
    pub async fn invoke(&self, state: S) -> Result<S, AgentError> {
        let mut state = state;
        for id in &self.edge_order {
            let node = self.nodes.get(id).expect("compiled graph has all nodes");
            state = node.run(state).await?;
        }
        Ok(state)
    }
}
