use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{MessageId, NotificationParams};

/// Parameters for notifications/cancelled.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CancelledNotificationParams {
    #[serde(flatten)]
    pub base: NotificationParams,
    #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<MessageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
