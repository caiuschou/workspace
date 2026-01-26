use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{NotificationParams, Progress, ProgressToken};

/// Parameters for notifications/progress.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProgressNotificationParams {
    #[serde(flatten)]
    pub base: NotificationParams,
    #[serde(flatten)]
    pub progress: Progress,
    #[serde(rename = "progressToken")]
    pub progress_token: ProgressToken,
}
