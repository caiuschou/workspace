//! Actor channel and reference.
//!
//! `AgentChannel` provides the inbox/outbox for an actor; `ActorRef<S>` is
//! a handle to send messages or perform request-response. Interacts with:
//! `ActorAgent`, `Handler`, `Supervisor`.

use std::marker::PhantomData;
use std::time::Duration;

use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot};

use super::types::{AgentMessage, Task, TaskResult};
use crate::error::ActorError;

/// Bidirectional channel: the sender may be cloned into multiple `ActorRef`s;
/// the receiver is consumed by `ActorAgent::run`.
///
/// Created by `AgentChannel::new(cap)`; use `split()` to obtain `ActorRef` and
/// receiver for the actor loop.
#[derive(Debug)]
pub struct AgentChannel {
    tx: mpsc::Sender<AgentMessage>,
    rx: mpsc::Receiver<AgentMessage>,
}

impl AgentChannel {
    /// Creates a channel with the given buffer capacity.
    pub fn new(cap: usize) -> Self {
        let (tx, rx) = mpsc::channel(cap);
        Self { tx, rx }
    }

    /// Splits into sender and receiver: `ActorRef` for callers, `Receiver` for
    /// `ActorAgent::run`. The type parameter `S` is the actor’s state type.
    pub fn split<S>(self) -> (ActorRef<S>, mpsc::Receiver<AgentMessage>) {
        (ActorRef::new(self.tx), self.rx)
    }
}

/// Handle to an actor used to send messages or perform request-response.
///
/// Type parameter `S` is the state type of the actor (for type-level tracking);
/// method behaviour does not depend on `S`. Used by `Supervisor` as
/// `ActorRef<()>` for workers. Interacts with: `AgentChannel`, `Supervisor::dispatch`.
#[derive(Clone, Debug)]
pub struct ActorRef<S> {
    tx: mpsc::Sender<AgentMessage>,
    _state: PhantomData<fn() -> S>,
}

impl<S> ActorRef<S> {
    /// Builds an `ActorRef` from the given sender.
    pub fn new(tx: mpsc::Sender<AgentMessage>) -> Self {
        Self {
            tx,
            _state: PhantomData,
        }
    }

    /// Sends a message without waiting for it to be processed.
    ///
    /// Returns `Ok(())` if the message was queued, or
    /// `Err(ActorError::ChannelClosed)` if the receiver was dropped.
    pub async fn send(&self, msg: AgentMessage) -> Result<(), ActorError> {
        self.tx.send(msg).await.map_err(|_| ActorError::ChannelClosed)
    }

    /// Tries to send without blocking; returns an error if the channel is full
    /// or closed.
    pub fn try_send(&self, msg: AgentMessage) -> Result<(), ActorError> {
        self.tx.try_send(msg).map_err(|e| match e {
            TrySendError::Full(_) => ActorError::HandleFailed("channel full".into()),
            TrySendError::Closed(_) => ActorError::ChannelClosed,
        })
    }

    /// Sends within the given timeout; returns `ActorError::SendTimeout` on
    /// expiry.
    pub async fn send_timeout(
        &self,
        msg: AgentMessage,
        timeout: Duration,
    ) -> Result<(), ActorError> {
        tokio::time::timeout(timeout, self.tx.send(msg))
            .await
            .map_err(|_| ActorError::SendTimeout)?
            .map_err(|_| ActorError::ChannelClosed)
    }

    /// Request-response: sends `Request(task, tx)` and awaits `TaskResult` (or
    /// error). The actor’s `Handler` must handle `AgentMessage::Request` and
    /// reply via the oneshot. Used by `Supervisor::dispatch`.
    pub async fn request(&self, task: Task) -> Result<TaskResult, ActorError> {
        let (tx, rx) = oneshot::channel();
        let msg = AgentMessage::Request(task, tx);
        self.tx.send(msg).await.map_err(|_| ActorError::ChannelClosed)?;
        rx.await.map_err(|_| ActorError::RequestFailed("response channel closed".into()))?
    }

    /// Request-response with a timeout for both send and receive; returns
    /// `ActorError::SendTimeout` if either step times out.
    pub async fn request_timeout(
        &self,
        task: Task,
        timeout: Duration,
    ) -> Result<TaskResult, ActorError> {
        let (tx, rx) = oneshot::channel();
        let msg = AgentMessage::Request(task, tx);
        tokio::time::timeout(timeout, self.tx.send(msg))
            .await
            .map_err(|_| ActorError::SendTimeout)?
            .map_err(|_| ActorError::ChannelClosed)?;
        tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| ActorError::SendTimeout)?
            .map_err(|_| ActorError::RequestFailed("response channel closed".into()))?
    }
}
