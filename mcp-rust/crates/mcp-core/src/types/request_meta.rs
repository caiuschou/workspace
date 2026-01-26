use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ProgressToken, RelatedTaskMetadata};

/// Metadata embedded in request/notification/result `_meta` fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct RequestMeta {
    /// Request progress token for out-of-band progress notifications.
    #[serde(rename = "progressToken", skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<ProgressToken>,
    /// Optional related task reference.
    #[serde(
        rename = "io.modelcontextprotocol/related-task",
        skip_serializing_if = "Option::is_none"
    )]
    pub related_task: Option<RelatedTaskMetadata>,
}
