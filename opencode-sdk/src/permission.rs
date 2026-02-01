//! Permission API for OpenCode Server.
//!
//! List and respond to permission requests.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

/// Request body for permission reply.
#[derive(Debug, Clone, Serialize)]
pub struct PermissionReplyRequest {
    /// Reply: "allow" or "deny".
    pub reply: String,
    /// Optional message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Client {
    /// Lists all pending permission requests.
    ///
    /// `GET /permission`
    pub async fn permission_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/permission", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<serde_json::Value> =
            serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }

    /// Approves or denies a permission request.
    ///
    /// `POST /permission/{requestID}/reply`
    pub async fn permission_reply(
        &self,
        request_id: &str,
        directory: Option<&Path>,
        body: PermissionReplyRequest,
    ) -> Result<bool, Error> {
        let url = format!("{}/permission/{}/reply", self.base_url(), request_id);
        let req = self.http().post(&url).json(&body).with_directory(directory);

        let response = req.send().await?;
        let value: bool = response.json().await?;
        Ok(value)
    }
}
