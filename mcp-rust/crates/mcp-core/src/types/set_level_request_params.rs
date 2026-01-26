use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{LoggingLevel, RequestParams};

/// Parameters for a logging/setLevel request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetLevelRequestParams {
    #[serde(flatten)]
    pub base: RequestParams,
    pub level: LoggingLevel,
}
