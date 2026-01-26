use async_trait::async_trait;
use futures::future::BoxFuture;

use mcp_core::protocol::{NotificationContext, NotificationHandler, ProtocolError};
use mcp_core::types::NotificationMessage;

/// Adapter to turn async closures into notification handlers.
pub struct NotificationHandlerFn<F> {
    handler: F,
}

impl<F> NotificationHandlerFn<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> NotificationHandler for NotificationHandlerFn<F>
where
    F: Send
        + Sync
        + 'static
        + Fn(
            &NotificationMessage,
            &NotificationContext,
        ) -> BoxFuture<'static, Result<(), ProtocolError>>,
{
    async fn handle(
        &self,
        notification: &NotificationMessage,
        context: &NotificationContext,
    ) -> Result<(), ProtocolError> {
        (self.handler)(notification, context).await
    }
}
