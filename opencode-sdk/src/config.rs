//! Config API for OpenCode Server.
//!
//! Get and update OpenCode configuration.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

impl Client {
    /// Gets the current OpenCode configuration.
    ///
    /// `GET /config`
    pub async fn config_get(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/config", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Updates OpenCode configuration.
    ///
    /// `PATCH /config`. Pass a JSON value with the fields to update.
    pub async fn config_patch(
        &self,
        directory: Option<&Path>,
        body: impl Serialize,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/config", self.base_url());
        let req = self.http().patch(&url).json(&body).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Lists configured AI providers and their default models.
    ///
    /// `GET /config/providers`
    pub async fn config_providers(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/config/providers", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }
}
