use async_trait::async_trait;

use crate::types::NotificationMessage;

use super::{NotificationContext, ProtocolError};

/// Handler that processes a single JSON-RPC-style notification.
#[async_trait]
pub trait NotificationHandler: Send + Sync + 'static {
    async fn handle(
        &self,
        notification: &NotificationMessage,
        context: &NotificationContext,
    ) -> Result<(), ProtocolError>;
}
