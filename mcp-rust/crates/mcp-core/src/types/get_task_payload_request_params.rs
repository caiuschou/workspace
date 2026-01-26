use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::RequestParams;

/// Parameters for tasks/result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GetTaskPayloadRequestParams {
    #[serde(flatten)]
    pub base: RequestParams,
    #[serde(rename = "taskId")]
    pub task_id: String,
}
