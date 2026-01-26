use std::sync::Arc;

use mcp_core::types::{ElicitRequestFormParams, ElicitRequestUrlParams, ElicitResult};

/// Error type for elicitation handler.
#[derive(Debug, Clone)]
pub struct ElicitationError(pub String);

impl std::fmt::Display for ElicitationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ElicitationError {}

/// Handler trait for elicitation/create requests (form mode, synchronous).
pub trait FormElicitationHandler: Send + Sync + 'static {
    /// Handle a form elicitation request from the server.
    fn handle(&self, params: ElicitRequestFormParams) -> Result<ElicitResult, ElicitationError>;
}

/// Handler trait for elicitation/create requests (URL mode, synchronous).
pub trait UrlElicitationHandler: Send + Sync + 'static {
    /// Handle a URL elicitation request from the server.
    fn handle(&self, params: ElicitRequestUrlParams) -> Result<ElicitResult, ElicitationError>;
}

/// Type alias for boxed form elicitation handler.
pub type BoxedFormElicitationHandler = Arc<dyn FormElicitationHandler>;

/// Type alias for boxed URL elicitation handler.
pub type BoxedUrlElicitationHandler = Arc<dyn UrlElicitationHandler>;

/// Function-based form elicitation handler implementation.
pub struct FormElicitationHandlerFn<F>(pub F);

impl<F> FormElicitationHandler for FormElicitationHandlerFn<F>
where
    F: Fn(ElicitRequestFormParams) -> Result<ElicitResult, ElicitationError> + Send + Sync + 'static,
{
    fn handle(&self, params: ElicitRequestFormParams) -> Result<ElicitResult, ElicitationError> {
        (self.0)(params)
    }
}

/// Function-based URL elicitation handler implementation.
pub struct UrlElicitationHandlerFn<F>(pub F);

impl<F> UrlElicitationHandler for UrlElicitationHandlerFn<F>
where
    F: Fn(ElicitRequestUrlParams) -> Result<ElicitResult, ElicitationError> + Send + Sync + 'static,
{
    fn handle(&self, params: ElicitRequestUrlParams) -> Result<ElicitResult, ElicitationError> {
        (self.0)(params)
    }
}
