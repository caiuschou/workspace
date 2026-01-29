//! Graph node trait: one step in a StateGraph.
//!
//! Receives state `S`, returns updated `S` (full or partial in future).
//! Used by `StateGraph` and `CompiledStateGraph`. Aligns with LangGraph node
//! `(state) -> partial`. Agents can implement `Node<S>` when `Agent::State == S`.

use async_trait::async_trait;

use crate::error::AgentError;

/// One step in a graph: state in, state out.
///
/// Used by `StateGraph` to run a single step. The graph runner passes the
/// result to the next node or returns. Aligns with LangGraph node
/// `(state) -> partial`; this minimal version returns full `S`.
///
/// **Interaction**: Implemented by graph nodes and by agents via blanket impl
/// when `Agent::State == S`. See `StateGraph::add_node` and `CompiledStateGraph::invoke`.
#[async_trait]
pub trait Node<S>: Send + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// Node id (e.g. `"chat"`, `"tool"`). Must be unique within a graph.
    fn id(&self) -> &str;

    /// One step: state in, state out.
    ///
    /// The graph runner passes the returned state to the next node or returns
    /// it as the final result. Currently full state only; partial updates
    /// (with merge) are a future extension.
    async fn run(&self, state: S) -> Result<S, AgentError>;
}
