use async_trait::async_trait;
use futures::future::BoxFuture;
use serde_json::Value;

use mcp_core::protocol::{ProtocolError, RequestContext, RequestHandler};
use mcp_core::types::RequestMessage;

/// Adapter to turn async closures into request handlers.
pub struct RequestHandlerFn<F> {
    handler: F,
}

impl<F> RequestHandlerFn<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> RequestHandler for RequestHandlerFn<F>
where
    F: Send
        + Sync
        + 'static
        + Fn(&RequestMessage, &RequestContext) -> BoxFuture<'static, Result<Value, ProtocolError>>,
{
    async fn handle(
        &self,
        request: &RequestMessage,
        context: &RequestContext,
    ) -> Result<Value, ProtocolError> {
        (self.handler)(request, context).await
    }
}
