//! Actor and channel (S6 multi-agent prototype).
//!
//! - `ActorId`, `Task`, `TaskResult`, `AgentMessage`: protocol types in `types`
//! - `Handler<S>`: message-handling trait used by `ActorAgent`
//! - `ActorAgent<S,H>`: message loop with inbox, state, handler
//! - `AgentChannel`, `ActorRef<S>`: channel and sender handle; `send` / `request`
//! - `SupervisionStrategy`, `Router`, `RoundRobinRouter`: supervision and routing in `supervise`
//! - Worker / Supervisor: see `crate::worker`, `crate::supervisor`

mod agent;
mod channel;
mod handler;
mod supervise;
mod types;

pub use agent::ActorAgent;
pub use channel::{AgentChannel, ActorRef};
pub use handler::Handler;
pub use supervise::{RoundRobinRouter, Router, SupervisionStrategy};
pub use types::{ActorId, AgentMessage, Task, TaskResult};
