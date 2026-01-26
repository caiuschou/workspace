use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::NotificationParams;

/// Parameters for notifications/resources/updated.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ResourceUpdatedNotificationParams {
    #[serde(flatten)]
    pub base: NotificationParams,
    pub uri: String,
}
