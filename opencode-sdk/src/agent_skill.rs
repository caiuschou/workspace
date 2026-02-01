//! Agent and Skill API for OpenCode Server.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use std::path::Path;

impl Client {
    /// Lists all available AI agents.
    ///
    /// `GET /agent`
    pub async fn agent_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/agent", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }

    /// Lists all available skills.
    ///
    /// `GET /skill`
    pub async fn skill_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/skill", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body).unwrap_or_default())
    }
}
