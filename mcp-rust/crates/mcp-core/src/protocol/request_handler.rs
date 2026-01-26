use async_trait::async_trait;
use serde_json::Value;

use crate::types::RequestMessage;

use super::{ProtocolError, RequestContext};

/// Handler that processes a single JSON-RPC-style request.
#[async_trait]
pub trait RequestHandler: Send + Sync + 'static {
    async fn handle(
        &self,
        request: &RequestMessage,
        context: &RequestContext,
    ) -> Result<Value, ProtocolError>;
}
