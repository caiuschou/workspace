use serde::{Deserialize, Serialize};

use crate::client::ResourceDefinition;

/// Result payload for resources/list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResourceListResult {
    pub resources: Vec<ResourceDefinition>,
}
