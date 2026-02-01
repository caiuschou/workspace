//! LSP, Formatter, and MCP API for OpenCode Server.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

impl Client {
    /// Gets LSP server status.
    ///
    /// `GET /lsp`
    pub async fn lsp_status(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/lsp", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }

    /// Gets formatter status.
    ///
    /// `GET /formatter`
    pub async fn formatter_status(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/formatter", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }

    /// Gets MCP server status.
    ///
    /// `GET /mcp`
    pub async fn mcp_status(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/mcp", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Adds an MCP server.
    ///
    /// `POST /mcp`. Body must have `name` and `config` (McpLocalConfig or McpRemoteConfig).
    pub async fn mcp_add(
        &self,
        directory: Option<&Path>,
        body: impl Serialize,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/mcp", self.base_url());
        let req = self.http().post(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Starts MCP OAuth flow.
    ///
    /// `POST /mcp/{name}/auth`
    pub async fn mcp_auth_start(
        &self,
        name: &str,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/mcp/{}/auth", self.base_url(), name);
        let req = self.http().post(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Removes MCP OAuth credentials.
    ///
    /// `DELETE /mcp/{name}/auth`
    pub async fn mcp_auth_remove(
        &self,
        name: &str,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/mcp/{}/auth", self.base_url(), name);
        let req = self.http().delete(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Starts MCP OAuth and waits for callback.
    ///
    /// `POST /mcp/{name}/auth/authenticate`
    pub async fn mcp_auth_authenticate(
        &self,
        name: &str,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/mcp/{}/auth/authenticate", self.base_url(), name);
        let req = self.http().post(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Completes MCP OAuth callback.
    ///
    /// `POST /mcp/{name}/auth/callback`
    pub async fn mcp_auth_callback(
        &self,
        name: &str,
        directory: Option<&Path>,
        code: &str,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/mcp/{}/auth/callback", self.base_url(), name);
        let req = self
            .http()
            .post(&url)
            .json(&serde_json::json!({ "code": code }))
            .with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Connects an MCP server.
    ///
    /// `POST /mcp/{name}/connect`
    pub async fn mcp_connect(
        &self,
        name: &str,
        directory: Option<&Path>,
    ) -> Result<bool, Error> {
        let url = format!("{}/mcp/{}/connect", self.base_url(), name);
        let req = self.http().post(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Disconnects an MCP server.
    ///
    /// `POST /mcp/{name}/disconnect`
    pub async fn mcp_disconnect(
        &self,
        name: &str,
        directory: Option<&Path>,
    ) -> Result<bool, Error> {
        let url = format!("{}/mcp/{}/disconnect", self.base_url(), name);
        let req = self.http().post(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }
}
