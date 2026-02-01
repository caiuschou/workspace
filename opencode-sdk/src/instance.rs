//! Instance API for OpenCode Server.
//!
//! Dispose the current OpenCode instance.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use std::path::Path;

impl Client {
    /// Disposes the current OpenCode instance, releasing all resources.
    ///
    /// `POST /instance/dispose`
    pub async fn instance_dispose(
        &self,
        directory: Option<&Path>,
    ) -> Result<bool, Error> {
        let url = format!("{}/instance/dispose", self.base_url());
        let req = self.http().post(&url).with_directory(directory);

        let response = req.send().await?;
        let result: bool = response.json().await?;
        Ok(result)
    }
}
