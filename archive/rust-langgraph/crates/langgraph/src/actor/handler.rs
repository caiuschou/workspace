//! Actor message-handling trait.
//!
//! `Handler<S>` is used by `ActorAgent` to process each `AgentMessage`;
//! implement it to define how an actor reacts to Task / Request / Stop / Ping.
//! Interacts with: `ActorAgent::run`, `AgentMessage`, `WorkerActor`.

use super::types::AgentMessage;
use crate::error::ActorError;

/// Logic that processes one message and may update actor state.
///
/// Implemented by adapters (e.g. `WorkerActor`) that drive a `Worker` or other
/// logic; invoked by `ActorAgent::run` in the message loop. For
/// `AgentMessage::Request(_, tx)`, the implementation must eventually call
/// `tx.send(result)` (or send an error) so the requester is unblocked.
pub trait Handler<S>: Send + Sync {
    /// Process a single message. `state` is owned by the caller and may be
    /// updated during handling. Returns an error to stop the actor loop.
    fn handle(&self, msg: AgentMessage, state: &mut S) -> Result<(), ActorError>;
}
