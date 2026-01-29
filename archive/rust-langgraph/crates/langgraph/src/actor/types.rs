//! Actor protocol types: identity, task, result, and messages.
//!
//! Used by `Handler`, `ActorRef`, `ActorAgent`, and worker/supervisor layers.

use tokio::sync::oneshot;

use crate::error::ActorError;

/// Unique identifier for an actor.
///
/// Used by `ActorAgent` and `ActorRef` for logging and routing.
/// Interacts with: `ActorAgent::id`, `ActorRef::request` (caller-side identity).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorId(pub String);

impl ActorId {
    /// Creates a new `ActorId` from the given string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Payload sent to a worker for processing.
///
/// Carried by `AgentMessage::Task` / `AgentMessage::Request`;
/// produced by `Supervisor::dispatch` and consumed by `Worker::handle`.
#[derive(Debug, Clone)]
pub struct Task {
    /// Optional task id for tracing; not used by routing in S6.
    pub id: Option<String>,
    /// Task content (e.g. Echo input string, research topic).
    pub payload: String,
}

impl Task {
    /// Builds a task with optional id and payload. `id` defaults to `None`.
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            id: None,
            payload: payload.into(),
        }
    }

    /// Sets the task id and returns `self` for chaining.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Result of a worker processing a task.
///
/// Returned by `Worker::handle` and by `ActorRef::request`;
/// used as the success type of `Supervisor::dispatch`.
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Whether the task was processed successfully.
    pub success: bool,
    /// Output content (main result on success, error description on failure).
    pub output: String,
}

impl TaskResult {
    /// Builds a successful result with the given output.
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
        }
    }

    /// Builds a failure result with the given message.
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            output: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_new_and_with_id() {
        let t = Task::new("hello");
        assert_eq!(t.payload, "hello");
        assert!(t.id.is_none());
        let t = t.with_id("id-1");
        assert_eq!(t.id.as_deref(), Some("id-1"));
    }

    #[test]
    fn task_result_ok_err() {
        let r = TaskResult::ok("done");
        assert!(r.success);
        assert_eq!(r.output, "done");
        let r = TaskResult::err("failed");
        assert!(!r.success);
        assert_eq!(r.output, "failed");
    }

    #[test]
    fn actor_id_display() {
        let id = ActorId::new("w1");
        assert_eq!(id.0, "w1");
        assert_eq!(format!("{id}"), "w1");
    }
}

/// Message sent to an actor’s inbox.
///
/// Handled by `Handler::handle` inside `ActorAgent`; sent via `ActorRef::send`
/// or `ActorRef::request`. `Request` carries a oneshot sender so the receiver
/// can reply with `TaskResult`. Interacts with: `ActorRef`, `Handler`, `ActorAgent::run`.
#[derive(Debug)]
pub enum AgentMessage {
    /// Fire-and-forget task; no reply expected.
    Task(Task),
    /// Request-response: handler must eventually send `TaskResult` (or error) on the oneshot.
    Request(Task, oneshot::Sender<std::result::Result<TaskResult, ActorError>>),
    /// Graceful stop; the actor should exit the loop after processing the current message.
    Stop,
    /// Liveness probe; if `Some(tx)` is present, reply with `()` on `tx`.
    Ping(Option<oneshot::Sender<()>>),
}
