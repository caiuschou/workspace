//! Find API for OpenCode Server.
//!
//! Text search, file search, and symbol search.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use std::path::Path;

impl Client {
    /// Searches for text patterns using ripgrep.
    ///
    /// `GET /find?pattern=...`
    pub async fn find_text(
        &self,
        pattern: &str,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/find", self.base_url());
        let req = self
            .http()
            .get(&url)
            .query(&[("pattern", pattern)])
            .with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<serde_json::Value> =
            serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }

    /// Searches for files by name or pattern.
    ///
    /// `GET /find/file?query=...`
    pub async fn find_file(
        &self,
        query: &str,
        directory: Option<&Path>,
        dirs: Option<&str>,
        file_type: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<String>, Error> {
        let url = format!("{}/find/file", self.base_url());
        let mut req = self
            .http()
            .get(&url)
            .query(&[("query", query)])
            .with_directory(directory);
        if let Some(d) = dirs {
            req = req.query(&[("dirs", d)]);
        }
        if let Some(t) = file_type {
            req = req.query(&[("type", t)]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<String> = serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }

    /// Searches for symbols (functions, classes, etc.) using LSP.
    ///
    /// `GET /find/symbol?query=...`
    pub async fn find_symbol(
        &self,
        query: &str,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/find/symbol", self.base_url());
        let req = self
            .http()
            .get(&url)
            .query(&[("query", query)])
            .with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<serde_json::Value> =
            serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }
}
