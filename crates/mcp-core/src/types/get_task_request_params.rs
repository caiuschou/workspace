use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::RequestParams;

/// Parameters for tasks/get.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GetTaskRequestParams {
    #[serde(flatten)]
    pub base: RequestParams,
    #[serde(rename = "taskId")]
    pub task_id: String,
}
