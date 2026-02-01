//! Session diff types for `GET /session/{sessionID}/diff`.

use serde::{Deserialize, Serialize};

/// A single diff item from `GET /session/{sessionID}/diff` (file change).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffItem {
    /// File path.
    pub path: Option<String>,
    /// Change type, e.g. `"added"`, `"modified"`, `"deleted"`.
    pub change_type: Option<String>,
}
