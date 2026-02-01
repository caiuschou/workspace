//! Project API for OpenCode Server.
//!
//! List, get current, and update projects.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Project information returned by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Project ID.
    pub id: Option<String>,
    /// Project name.
    pub name: Option<String>,
    /// Project icon.
    pub icon: Option<serde_json::Value>,
    /// Project commands.
    pub commands: Option<serde_json::Value>,
}

/// Request body for PATCH /project/{projectID}.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    /// Project name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Project icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<serde_json::Value>,
    /// Project commands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<serde_json::Value>,
}

impl Client {
    /// Lists all projects that have been opened with OpenCode.
    ///
    /// `GET /project`
    pub async fn project_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/project", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<serde_json::Value> =
            serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }

    /// Gets the currently active project.
    ///
    /// `GET /project/current`
    pub async fn project_current(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/project/current", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Updates project properties (name, icon, commands).
    ///
    /// `PATCH /project/{projectID}`
    pub async fn project_update(
        &self,
        project_id: &str,
        directory: Option<&Path>,
        request: UpdateProjectRequest,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/project/{}", self.base_url(), project_id);
        let req = self.http().patch(&url).json(&request).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }
}
