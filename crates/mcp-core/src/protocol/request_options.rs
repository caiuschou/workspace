use std::time::Duration;

use super::CancellationToken;

/// Per-request options for handling timeouts and cancellation.
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub timeout: Option<Duration>,
    pub cancel_token: Option<CancellationToken>,
}
