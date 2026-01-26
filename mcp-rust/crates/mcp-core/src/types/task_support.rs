use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Task execution preference for tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskSupport {
    Required,
    Optional,
    Forbidden,
}
