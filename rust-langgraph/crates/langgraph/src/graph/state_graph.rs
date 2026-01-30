//! State graph: nodes + linear edge order.
//!
//! Add nodes with `add_node`, define the chain with `add_edge`, then `compile`
//! or `compile_with_checkpointer` to get a `CompiledStateGraph`. Design: 16-memory-design.md.

use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::compile_error::CompilationError;
use crate::graph::compiled::CompiledStateGraph;
use crate::graph::node::Node;
use crate::memory::Checkpointer;

/// State graph: nodes plus linear edge order. No conditional edges in minimal version.
///
/// Generic over state type `S`. Build with `add_node` / `add_edge`, then
/// `compile()` to obtain an executable graph.
///
/// **Interaction**: Accepts `Box<dyn Node<S>>`; produces `CompiledStateGraph<S>`.
pub struct StateGraph<S> {
    nodes: HashMap<String, Box<dyn Node<S>>>,
    /// Linear chain: [id1, id2, ...] => START -> id1 -> id2 -> ... -> END
    edge_order: Vec<String>,
}

impl<S> Default for StateGraph<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> StateGraph<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates an empty graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edge_order: Vec::new(),
        }
    }

    /// Adds a node; id must be unique. Replaces if same id.
    ///
    /// Returns `&mut Self` for method chaining. The node is stored as
    /// `Box<dyn Node<S>>`; use `add_edge` to include it in the chain.
    pub fn add_node(&mut self, id: impl Into<String>, node: Box<dyn Node<S>>) -> &mut Self {
        self.nodes.insert(id.into(), node);
        self
    }

    /// Appends an edge from the current chain end to this node.
    ///
    /// Order of `add_edge` calls defines the chain: first is START→id, last
    /// leads to END. The given `to_id` must be registered via `add_node`
    /// before `compile()`.
    pub fn add_edge(&mut self, to_id: impl Into<String>) -> &mut Self {
        self.edge_order.push(to_id.into());
        self
    }

    /// Builds the executable graph: validates that all edge targets are registered nodes.
    ///
    /// Returns `CompilationError::NodeNotFound(id)` if any id in the edge order
    /// is not in the node map. On success, the graph is immutable and ready for `invoke`.
    pub fn compile(self) -> Result<CompiledStateGraph<S>, CompilationError> {
        self.compile_with_checkpointer_opt(None)
    }

    /// Builds the executable graph with a checkpointer for persistence (thread_id in config).
    ///
    /// Aligns with LangGraph `graph.compile(checkpointer=checkpointer)`. When `invoke(state, config)`
    /// is called with `config.thread_id`, the final state is saved after the run. Design: 16-memory-design.md §4.1.
    pub fn compile_with_checkpointer(
        self,
        checkpointer: Arc<dyn Checkpointer<S>>,
    ) -> Result<CompiledStateGraph<S>, CompilationError> {
        self.compile_with_checkpointer_opt(Some(checkpointer))
    }

    fn compile_with_checkpointer_opt(
        self,
        checkpointer: Option<Arc<dyn Checkpointer<S>>>,
    ) -> Result<CompiledStateGraph<S>, CompilationError> {
        for id in &self.edge_order {
            if !self.nodes.contains_key(id) {
                return Err(CompilationError::NodeNotFound(id.clone()));
            }
        }
        Ok(CompiledStateGraph {
            nodes: self.nodes,
            edge_order: self.edge_order,
            checkpointer,
        })
    }
}
