use async_trait::async_trait;
use serde_json::Value;

use mcp_core::protocol::RequestContext;
use mcp_core::types::CallToolResult;

use crate::server::ServerError;

/// Handler for tool execution.
#[async_trait]
pub trait ToolHandler: Send + Sync + 'static {
    async fn call(
        &self,
        arguments: Option<Value>,
        context: RequestContext,
    ) -> Result<CallToolResult, ServerError>;
}

#[async_trait]
impl<F, Fut> ToolHandler for F
where
    F: Send + Sync + 'static + Fn(Option<Value>, RequestContext) -> Fut,
    Fut: std::future::Future<Output = Result<CallToolResult, ServerError>> + Send,
{
    async fn call(
        &self,
        arguments: Option<Value>,
        context: RequestContext,
    ) -> Result<CallToolResult, ServerError> {
        (self)(arguments, context).await
    }
}
