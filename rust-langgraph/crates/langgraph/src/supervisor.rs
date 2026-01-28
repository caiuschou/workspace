//! Supervisor: holds a set of worker `ActorRef`s and a `Router`; receives tasks,
//! dispatches to a worker, and returns `TaskResult`.
//!
//! Minimal S6 implementation: dispatch to one or more workers by index chosen
//! via `Router`. Interacts with: `ActorRef`, `Router`, `Task`, `TaskResult`, worker actors.

use crate::actor::{ActorRef, Router, Task, TaskResult};
use crate::error::ActorError;

/// Holds worker references and a routing strategy; receives a task, picks a
/// worker via the router, and returns the result of `ActorRef::request(task)`.
///
/// Workers are `ActorRef<()>` (state type is unit in S6). Interacts with:
/// `Router::route`, `ActorRef::request`, worker actors running `WorkerActor`.
pub struct Supervisor<R> {
    /// List of worker references; indices match `Router::route` output.
    workers: Vec<ActorRef<()>>,
    /// Routing strategy used by `dispatch` to choose a worker index.
    router: R,
}

impl<R> Supervisor<R>
where
    R: Router,
{
    /// Creates a supervisor with the given router; workers are added via
    /// `with_workers` or `add_worker`.
    pub fn new(router: R) -> Self {
        Self {
            workers: Vec::new(),
            router,
        }
    }

    /// Sets the worker list (replaces any existing). Returns `self` for chaining.
    pub fn with_workers(mut self, workers: Vec<ActorRef<()>>) -> Self {
        self.workers = workers;
        self
    }

    /// Appends one worker to the list.
    pub fn add_worker(&mut self, w: ActorRef<()>) {
        self.workers.push(w);
    }

    /// Dispatches a task: uses `Router::route` to choose a worker index, then
    /// calls `request(task)` on that worker and returns the result. Fails if
    /// there are no workers or the router returns `None`.
    pub async fn dispatch(&self, task: Task) -> Result<TaskResult, ActorError> {
        let idx = self
            .router
            .route(&task, self.workers.len())
            .ok_or_else(|| ActorError::RequestFailed("no workers".into()))?;
        let worker = self
            .workers
            .get(idx)
            .ok_or_else(|| ActorError::RequestFailed("worker index out of range".into()))?;
        worker.request(task).await
    }
}
