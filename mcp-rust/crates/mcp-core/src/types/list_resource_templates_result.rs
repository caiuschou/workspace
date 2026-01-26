use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{PaginatedResult, ResourceTemplate};

/// Result for resources/templates/list.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ListResourceTemplatesResult {
    #[serde(flatten)]
    pub pagination: PaginatedResult,
    #[serde(rename = "resourceTemplates")]
    pub resource_templates: Vec<ResourceTemplate>,
}
