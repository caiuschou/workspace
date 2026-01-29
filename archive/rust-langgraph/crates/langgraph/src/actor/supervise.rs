//! Supervision strategy and routing (minimal S6 implementation).
//!
//! - `SupervisionStrategy`: who is affected on failure (placeholder; restart logic later)
//! - `Router`: selects a worker index for a given task
//! - `RoundRobinRouter`: cycles through worker indices. Interacts with: `Supervisor::dispatch`, `Task`.

use super::Task;

/// Who is restarted or affected when a child fails. Placeholder in S6; restart
/// logic to be implemented later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionStrategy {
    /// Restart only the failed child.
    OneForOne,
    /// Restart all nodes in the group (group defined by caller).
    OneForAll,
    /// Restart all.
    AllForOne,
}

/// Selects which worker (by index) handles a task.
///
/// Used by `Supervisor::dispatch` to pick the worker index; returns `None` when
/// no worker is available (e.g. `workers_len == 0`). Interacts with: `Task`, `Supervisor`.
pub trait Router: Send + Sync {
    /// Returns an index in `[0..workers_len)`, or `None` if no worker is available.
    fn route(&self, task: &Task, workers_len: usize) -> Option<usize>;
}

/// Round-robin router: selects indices in order 0, 1, …, n-1, 0, … on each call.
#[derive(Debug, Default)]
pub struct RoundRobinRouter {
    next: std::sync::atomic::AtomicUsize,
}

impl RoundRobinRouter {
    /// Creates a new round-robin router.
    pub fn new() -> Self {
        Self {
            next: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Router for RoundRobinRouter {
    fn route(&self, _task: &Task, workers_len: usize) -> Option<usize> {
        if workers_len == 0 {
            return None;
        }
        let n = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(n % workers_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_cycles() {
        let r = RoundRobinRouter::new();
        assert_eq!(r.route(&Task::new("a"), 0), None);
        assert_eq!(r.route(&Task::new("a"), 3), Some(0));
        assert_eq!(r.route(&Task::new("a"), 3), Some(1));
        assert_eq!(r.route(&Task::new("a"), 3), Some(2));
        assert_eq!(r.route(&Task::new("a"), 3), Some(0));
    }
}
