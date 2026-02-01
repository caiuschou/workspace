//! Logging API for OpenCode Server.
//!
//! Write log entries to the server. (Distinct from [crate::log] which configures SDK logging.)

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

/// Request body for `POST /log` (write log entry to server).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryRequest {
    /// Service name for the log entry.
    pub service: String,
    /// Log level (e.g. "info", "warn", "error").
    pub level: String,
    /// Log message.
    pub message: String,
    /// Additional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl Client {
    /// Writes a log entry to the server.
    ///
    /// `POST /log`
    ///
    /// # Errors
    ///
    /// Returns `Err` when the HTTP request fails or response JSON cannot be parsed.
    pub async fn log_write(
        &self,
        directory: Option<&Path>,
        request: LogEntryRequest,
    ) -> Result<bool, Error> {
        let url = format!("{}/log", self.base_url());
        let req = self.http().post(&url).json(&request).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }
}
