use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Cursor, RequestMeta};

/// Base result payload for paginated responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct PaginatedResult {
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}
