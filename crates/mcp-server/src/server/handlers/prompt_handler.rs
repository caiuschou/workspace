use std::collections::HashMap;

use async_trait::async_trait;

use mcp_core::protocol::RequestContext;
use mcp_core::types::GetPromptResult;

use crate::server::ServerError;

/// Handler for prompts/get requests.
#[async_trait]
pub trait PromptHandler: Send + Sync + 'static {
    async fn get(
        &self,
        arguments: Option<HashMap<String, String>>,
        context: RequestContext,
    ) -> Result<GetPromptResult, ServerError>;
}

#[async_trait]
impl<F, Fut> PromptHandler for F
where
    F: Send + Sync + 'static + Fn(Option<HashMap<String, String>>, RequestContext) -> Fut,
    Fut: std::future::Future<Output = Result<GetPromptResult, ServerError>> + Send,
{
    async fn get(
        &self,
        arguments: Option<HashMap<String, String>>,
        context: RequestContext,
    ) -> Result<GetPromptResult, ServerError> {
        (self)(arguments, context).await
    }
}
