//! Minimal agent trait: state in, state out.
//!
//! Aligns with LangGraph: no separate Input/Output; invoke(state) returns updated state.
//! Used by all agents (e.g. EchoAgent) and by callers that run one step per `run(state)`.

use async_trait::async_trait;

use crate::error::AgentError;

/// Minimal agent: state in, state out. Aligns with LangGraph (no Input/Output).
///
/// One step: receive state, return updated state. Equivalent to a single node
/// with fixed edge START → node → END. No streaming or tools in this minimal API.
///
/// **State is defined by the implementer**: each agent chooses its own `State` type
/// and fields (e.g. `messages` only, or `messages` + `metadata`, or a custom struct).
/// See the echo example for a minimal `AgentState` (message list) defined in the example.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Display name of the agent (e.g. "echo", "chat").
    fn name(&self) -> &str;

    /// State type for this agent; **defined by the implementer** (fields and shape).
    /// Must be cloneable and sendable across async boundaries.
    type State: Clone + Send + Sync + 'static;

    /// One step: receive state, return updated state.
    ///
    /// Caller puts input (e.g. user message) into state before calling;
    /// reads output (e.g. assistant message) from the returned state.
    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError>;
}
