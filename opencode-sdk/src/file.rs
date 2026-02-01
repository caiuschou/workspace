//! File API for OpenCode Server.
//!
//! List files and get file status (git status) in the project directory.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use std::path::Path;

impl Client {
    /// Lists files and directories at the given path in the project.
    ///
    /// `GET /file?path=...`. Returns an array of file/directory entries (structure is server-defined).
    pub async fn file_list(
        &self,
        path: &str,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/file", self.base_url());
        let req = self
            .http()
            .get(&url)
            .query(&[("path", path)])
            .with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!([]));
        Ok(value)
    }

    /// Reads file content at the given path.
    ///
    /// `GET /file/content?path=...`
    pub async fn file_content(
        &self,
        path: &str,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/file/content", self.base_url());
        let req = self
            .http()
            .get(&url)
            .query(&[("path", path)])
            .with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Gets git status of all files in the project.
    ///
    /// `GET /file/status`. Returns an array of file status entries (structure is server-defined).
    pub async fn file_status(&self, directory: Option<&Path>) -> Result<serde_json::Value, Error> {
        let url = format!("{}/file/status", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!([]));
        Ok(value)
    }
}
