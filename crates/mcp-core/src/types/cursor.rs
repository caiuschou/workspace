use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Opaque pagination cursor.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Cursor(pub String);

impl From<&str> for Cursor {
    fn from(value: &str) -> Self {
        Cursor(value.to_string())
    }
}

impl From<String> for Cursor {
    fn from(value: String) -> Self {
        Cursor(value)
    }
}

impl AsRef<str> for Cursor {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
