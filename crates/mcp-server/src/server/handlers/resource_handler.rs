use async_trait::async_trait;

use mcp_core::protocol::RequestContext;
use mcp_core::types::ReadResourceResult;

use crate::server::ServerError;

/// Handler for reading resources.
#[async_trait]
pub trait ResourceHandler: Send + Sync + 'static {
    async fn read(
        &self,
        uri: String,
        context: RequestContext,
    ) -> Result<ReadResourceResult, ServerError>;
}

#[async_trait]
impl<F, Fut> ResourceHandler for F
where
    F: Send + Sync + 'static + Fn(String, RequestContext) -> Fut,
    Fut: std::future::Future<Output = Result<ReadResourceResult, ServerError>> + Send,
{
    async fn read(
        &self,
        uri: String,
        context: RequestContext,
    ) -> Result<ReadResourceResult, ServerError> {
        (self)(uri, context).await
    }
}
