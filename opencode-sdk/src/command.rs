//! Command API for OpenCode Server.
//!
//! List available commands.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use std::path::Path;

impl Client {
    /// Lists all available commands.
    ///
    /// `GET /command`
    pub async fn command_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/command", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<serde_json::Value> =
            serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }
}
