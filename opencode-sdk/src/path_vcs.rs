//! Path and VCS API for OpenCode Server.
//!
//! Get working directory and version control info.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use std::path::Path;

impl Client {
    /// Gets the current working directory and related path information.
    ///
    /// `GET /path`
    pub async fn path_get(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/path", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Gets VCS (e.g. git) information for the current project.
    ///
    /// `GET /vcs`
    pub async fn vcs_get(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/vcs", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }
}
