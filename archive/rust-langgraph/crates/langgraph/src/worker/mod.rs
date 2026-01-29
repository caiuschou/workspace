//! Worker and actor-driven adapter (S6).
//!
//! - `Worker`: async process `Task` and return `TaskResult`
//! - `EchoWorker`: minimal implementation that returns the payload unchanged
//! - `WorkerActor`: wraps a `Worker` as `Handler<()>` for use by `ActorAgent`.
//!   Interacts with: `ActorAgent`, `Handler`, `Task`, `TaskResult`, `Supervisor`.

use std::sync::Arc;

use async_trait::async_trait;

use crate::actor::{AgentMessage, Handler, Task, TaskResult};
use crate::error::ActorError;

/// Async processing of a task: name, description, and `handle(Task) -> TaskResult`.
///
/// Implemented by `EchoWorker` and later by `ResearcherWorker` etc.; used by
/// `WorkerActor` to execute tasks in the actor loop. Interacts with: `Task`,
/// `TaskResult`, `WorkerActor`.
#[async_trait]
pub trait Worker: Send + Sync {
    /// Worker display name.
    fn name(&self) -> &str;

    /// Short description for routing or logging; default is empty.
    fn description(&self) -> &str {
        ""
    }

    /// Process one task asynchronously and return the result.
    async fn handle(&self, task: Task) -> TaskResult;
}

/// Minimal worker: returns `Task::payload` unchanged as a successful `TaskResult`.
#[derive(Debug, Default)]
pub struct EchoWorker;

impl EchoWorker {
    /// Creates a new `EchoWorker`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Worker for EchoWorker {
    fn name(&self) -> &str {
        "EchoWorker"
    }

    fn description(&self) -> &str {
        "Echoes the task payload as the result"
    }

    async fn handle(&self, task: Task) -> TaskResult {
        TaskResult::ok(task.payload)
    }
}

/// Wraps a `Worker` as `Handler<()>` so it can be driven by `ActorAgent`.
///
/// On `AgentMessage::Request(task, tx)`, runs `worker.handle(task)` (via
/// `block_on` in the current runtime) and sends the result on `tx`. For
/// fire-and-forget `Task`, the result is computed but discarded. `Stop` and
/// `Ping` are ignored. Interacts with: `ActorAgent`, `Handler`, `AgentMessage`.
pub struct WorkerActor<W> {
    worker: Arc<W>,
}

impl<W> WorkerActor<W>
where
    W: Worker + 'static,
{
    /// Builds a `WorkerActor` that delegates all Task/Request messages to `worker`.
    pub fn new(worker: Arc<W>) -> Self {
        Self { worker }
    }
}

impl<W> Handler<()> for WorkerActor<W>
where
    W: Worker + 'static,
{
    fn handle(&self, msg: AgentMessage, _state: &mut ()) -> Result<(), ActorError> {
        let (task, reply) = match msg {
            AgentMessage::Task(t) => (t, None),
            AgentMessage::Request(t, tx) => (t, Some(tx)),
            AgentMessage::Stop | AgentMessage::Ping(_) => return Ok(()),
        };
        // Run async worker in the current runtime so the sync Handler can wait for it.
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.worker.handle(task))
        });
        if let Some(tx) = reply {
            let _ = tx.send(Ok(result));
        }
        Ok(())
    }
}
