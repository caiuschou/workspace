//! Actor message loop.
//!
//! `ActorAgent<S,H>` runs a loop that reads from an inbox and dispatches to
//! `Handler<S>`; used to host workers or other actor logic. Interacts with:
//! `AgentChannel::split`, `Handler`, `WorkerActor`.

use tokio::sync::mpsc;

use super::handler::Handler;
use super::types::{AgentMessage, ActorId};
use crate::error::ActorError;

/// Drives the message loop: holds id, inbox, state, and handler; `run()` receives
/// messages and calls `Handler::handle` until `Stop` or channel close.
///
/// The receiver is typically from `AgentChannel::split()`. Used to run a
/// `WorkerActor` or other handler.
pub struct ActorAgent<S, H> {
    /// Actor identity; used for logging and routing.
    pub id: ActorId,
    /// Inbox; usually the receiver from `AgentChannel::split()`.
    inbox: mpsc::Receiver<AgentMessage>,
    /// Current state; readable and writable by `Handler::handle`.
    pub state: S,
    /// Message-handling logic; e.g. `WorkerActor<W>`.
    handler: H,
}

impl<S, H> ActorAgent<S, H>
where
    S: Send + 'static,
    H: Handler<S> + Send + 'static,
{
    /// Builds an actor. `inbox` should be the receiver from `AgentChannel::split()`.
    pub fn new(id: ActorId, inbox: mpsc::Receiver<AgentMessage>, state: S, handler: H) -> Self {
        Self {
            id,
            inbox,
            state,
            handler,
        }
    }

    /// Runs the message loop until `AgentMessage::Stop` is received or the
    /// inbox is closed. Returns `Ok(())` on normal exit; propagates handler
    /// errors.
    pub async fn run(&mut self) -> Result<(), ActorError> {
        while let Some(msg) = self.inbox.recv().await {
            match msg {
                AgentMessage::Stop => break,
                m => self.handler.handle(m, &mut self.state)?,
            }
        }
        Ok(())
    }
}
