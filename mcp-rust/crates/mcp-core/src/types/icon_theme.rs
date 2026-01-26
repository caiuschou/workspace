use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Theme hint for icons.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IconTheme {
    Light,
    Dark,
}
