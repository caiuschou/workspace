use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::RequestParams;

/// Base parameters for resource requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ResourceRequestParams {
    #[serde(flatten)]
    pub base: RequestParams,
    pub uri: String,
}
