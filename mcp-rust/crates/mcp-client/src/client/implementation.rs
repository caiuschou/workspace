use serde::{Deserialize, Serialize};

/// Identifies an MCP implementation by name and optional version.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Implementation {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Implementation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}
