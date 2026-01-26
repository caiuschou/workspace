use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::RequestMeta;

/// Base parameters shared by requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct RequestParams {
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
}
