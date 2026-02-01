//! Experimental API for OpenCode Server.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use std::path::Path;

impl Client {
    /// Lists all tool IDs.
    ///
    /// `GET /experimental/tool/ids`
    pub async fn experimental_tool_ids(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/experimental/tool/ids", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Gets tools for a provider+model.
    ///
    /// `GET /experimental/tool?provider=...&model=...`
    pub async fn experimental_tool_list(
        &self,
        provider: &str,
        model: &str,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/experimental/tool", self.base_url());
        let req = self
            .http()
            .get(&url)
            .query(&[("provider", provider), ("model", model)])
            .with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Gets MCP resources.
    ///
    /// `GET /experimental/resource`
    pub async fn experimental_resource_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/experimental/resource", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Lists worktrees.
    ///
    /// `GET /experimental/worktree`
    pub async fn experimental_worktree_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/experimental/worktree", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }

    /// Creates a worktree.
    ///
    /// `POST /experimental/worktree`
    pub async fn experimental_worktree_create(
        &self,
        directory: Option<&Path>,
        name: Option<&str>,
        start_command: Option<&str>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/experimental/worktree", self.base_url());
        let mut body = serde_json::Map::new();
        if let Some(n) = name {
            body.insert("name".to_string(), serde_json::Value::String(n.to_string()));
        }
        if let Some(s) = start_command {
            body.insert("startCommand".to_string(), serde_json::Value::String(s.to_string()));
        }
        let req = self.http().post(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Removes a worktree.
    ///
    /// `DELETE /experimental/worktree`
    pub async fn experimental_worktree_remove(
        &self,
        directory: Option<&Path>,
        worktree_directory: &str,
    ) -> Result<bool, Error> {
        let url = format!("{}/experimental/worktree", self.base_url());
        let req = self
            .http()
            .delete(&url)
            .json(&serde_json::json!({ "directory": worktree_directory }))
            .with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Resets a worktree branch.
    ///
    /// `POST /experimental/worktree/reset`
    pub async fn experimental_worktree_reset(
        &self,
        directory: Option<&Path>,
        worktree_directory: &str,
    ) -> Result<bool, Error> {
        let url = format!("{}/experimental/worktree/reset", self.base_url());
        let req = self
            .http()
            .post(&url)
            .json(&serde_json::json!({ "directory": worktree_directory }))
            .with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }
}
