//! Shared HTTP request helpers for OpenCode SDK.
//!
//! Provides `RequestBuilderExt::with_directory` so API modules can attach
//! the `directory` query parameter without repeating the same pattern.

use reqwest::RequestBuilder;
use std::path::Path;

/// Extension trait for `reqwest::RequestBuilder` to add the `directory` query parameter.
///
/// Used by event, file, session, and other API modules that accept an optional
/// project directory.
pub(crate) trait RequestBuilderExt {
    /// Appends `directory` as a query parameter when `directory` is `Some` and UTF-8.
    fn with_directory(self, directory: Option<&Path>) -> Self;
}

impl RequestBuilderExt for RequestBuilder {
    fn with_directory(mut self, directory: Option<&Path>) -> Self {
        if let Some(dir) = directory.and_then(|p| p.to_str()) {
            self = self.query(&[("directory", dir)]);
        }
        self
    }
}
