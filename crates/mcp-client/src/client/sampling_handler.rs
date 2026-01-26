use std::sync::Arc;

use mcp_core::types::{CreateMessageRequestParams, CreateMessageResult};

/// Error type for sampling handler.
#[derive(Debug, Clone)]
pub struct SamplingError(pub String);

impl std::fmt::Display for SamplingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SamplingError {}

/// Handler trait for sampling/createMessage requests (synchronous).
pub trait SamplingHandler: Send + Sync + 'static {
    /// Handle a sampling request from the server.
    fn handle(&self, params: CreateMessageRequestParams) -> Result<CreateMessageResult, SamplingError>;
}

/// Type alias for boxed sampling handler.
pub type BoxedSamplingHandler = Arc<dyn SamplingHandler>;

/// Function-based sampling handler implementation.
pub struct SamplingHandlerFn<F>(pub F);

impl<F> SamplingHandler for SamplingHandlerFn<F>
where
    F: Fn(CreateMessageRequestParams) -> Result<CreateMessageResult, SamplingError> + Send + Sync + 'static,
{
    fn handle(&self, params: CreateMessageRequestParams) -> Result<CreateMessageResult, SamplingError> {
        (self.0)(params)
    }
}
