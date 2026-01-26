use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Icon;

/// Optional icon set for metadata objects.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct Icons {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<Icon>>,
}
