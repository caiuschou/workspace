//! File API types for OpenCode Server (Serve API 12-file).

use serde::{Deserialize, Serialize};

/// A file or directory entry from `GET /file` (Serve API 12-file).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// Display name.
    pub name: Option<String>,
    /// Path relative to project root.
    pub path: Option<String>,
    /// Entry type, e.g. `"file"` or `"directory"`.
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    /// File size in bytes (if file).
    pub size: Option<u64>,
}

/// A file status entry from `GET /file/status` (git status).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStatus {
    /// File path.
    pub path: Option<String>,
    /// Git status, e.g. `"modified"`, `"added"`, `"deleted"`.
    pub status: Option<String>,
}
