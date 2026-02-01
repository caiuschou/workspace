//! PTY (pseudo-terminal) API for OpenCode Server.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

/// Request body for creating a PTY session.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePtyRequest {
    /// Command to run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Command arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Working directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Session title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Environment variables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<serde_json::Value>,
}

/// Request body for updating a PTY session.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePtyRequest {
    /// Session title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Terminal size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<serde_json::Value>,
}

impl Client {
    /// Lists all PTY sessions.
    ///
    /// `GET /pty`
    pub async fn pty_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/pty", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }

    /// Creates a PTY session.
    ///
    /// `POST /pty`
    pub async fn pty_create(
        &self,
        directory: Option<&Path>,
        request: CreatePtyRequest,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/pty", self.base_url());
        let req = self.http().post(&url).json(&request).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Gets a PTY session by ID.
    ///
    /// `GET /pty/{ptyID}`
    pub async fn pty_get(
        &self,
        pty_id: &str,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/pty/{}", self.base_url(), pty_id);
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Updates a PTY session.
    ///
    /// `PUT /pty/{ptyID}`
    pub async fn pty_update(
        &self,
        pty_id: &str,
        directory: Option<&Path>,
        request: UpdatePtyRequest,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/pty/{}", self.base_url(), pty_id);
        let req = self.http().put(&url).json(&request).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Removes and terminates a PTY session.
    ///
    /// `DELETE /pty/{ptyID}`
    pub async fn pty_remove(
        &self,
        pty_id: &str,
        directory: Option<&Path>,
    ) -> Result<bool, Error> {
        let url = format!("{}/pty/{}", self.base_url(), pty_id);
        let req = self.http().delete(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Returns the WebSocket URL for connecting to a PTY session.
    ///
    /// `GET /pty/{ptyID}/connect` establishes a WebSocket connection. Use a WebSocket
    /// client to connect to the returned URL.
    pub fn pty_connect_url(&self, pty_id: &str, directory: Option<&Path>) -> String {
        let base = self.base_url();
        let ws_base = base
            .replacen("http://", "ws://", 1)
            .replacen("https://", "wss://", 1);
        let mut url = format!("{}/pty/{}/connect", ws_base, pty_id);
        if let Some(dir) = directory.and_then(|d| d.to_str()) {
            url.push_str(&format!("?directory={}", dir));
        }
        url
    }
}
