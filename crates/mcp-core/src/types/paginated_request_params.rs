use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Cursor, RequestParams};

/// Base parameters for paginated requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct PaginatedRequestParams {
    #[serde(flatten)]
    pub base: RequestParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}
