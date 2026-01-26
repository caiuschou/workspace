use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A request to include context from MCP servers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IncludeContext {
    /// Do not include any context.
    None,
    /// Include context from this server only.
    ThisServer,
    /// Include context from all connected servers.
    AllServers,
}

impl Default for IncludeContext {
    fn default() -> Self {
        Self::None
    }
}
