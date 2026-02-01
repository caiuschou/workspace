//! Project directory type for API calls.
//!
//! Use `ProjectDirectory::none()` for server's current working directory,
//! or `ProjectDirectory::from_path(path)` for an explicit project path.
//! Reduces `Option<&Path>` parameter count and improves discoverability.

use std::path::{Path, PathBuf};

/// Project directory for API calls.
///
/// Use [`none`](Self::none) for server's cwd; use [`from_path`](Self::from_path) for an explicit project.
/// Pass [`as_path`](Self::as_path) to API methods that accept `directory: Option<&Path>`.
///
/// # Examples
///
/// ```
/// use opencode_sdk::ProjectDirectory;
///
/// let none = ProjectDirectory::none();
/// let proj = ProjectDirectory::from_path("/path/to/project");
/// assert!(proj.as_path().is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct ProjectDirectory(Option<PathBuf>);

impl ProjectDirectory {
    /// No project directory (use server's current working directory).
    pub fn none() -> Self {
        Self(None)
    }

    /// Explicit project path.
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self(Some(path.into()))
    }

    /// Returns the path as `Option<&Path>` for API methods.
    pub fn as_path(&self) -> Option<&Path> {
        self.0.as_deref()
    }

    /// Returns true if a project path is set.
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl From<Option<PathBuf>> for ProjectDirectory {
    fn from(value: Option<PathBuf>) -> Self {
        Self(value)
    }
}

impl From<PathBuf> for ProjectDirectory {
    fn from(value: PathBuf) -> Self {
        Self(Some(value))
    }
}
