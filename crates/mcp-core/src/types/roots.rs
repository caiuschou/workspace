use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::Root;

/// Result payload for `roots/list`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct ListRootsResult {
    pub roots: Vec<Root>,
}
