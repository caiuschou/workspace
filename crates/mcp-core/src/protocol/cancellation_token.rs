use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use futures::future::poll_fn;
use futures::task::AtomicWaker;

/// Cooperative cancellation token for request handling.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
    waker: Arc<AtomicWaker>,
}

impl CancellationToken {
    /// Returns true if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Request cancellation and wake any waiters.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.waker.wake();
    }

    /// Future that resolves when cancellation is requested.
    pub async fn cancelled(&self) {
        let cancelled = &self.cancelled;
        let waker = &self.waker;
        poll_fn(|cx| {
            waker.register(cx.waker());
            if cancelled.load(Ordering::SeqCst) {
                std::task::Poll::Ready(())
            } else {
                std::task::Poll::Pending
            }
        })
        .await
    }
}
