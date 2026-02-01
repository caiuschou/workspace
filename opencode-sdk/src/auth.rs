//! Auth API for OpenCode Server.
//!
//! Set authentication credentials for providers.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

impl Client {
    /// Sets authentication credentials for a provider.
    ///
    /// `PUT /auth/{providerID}`. Pass the credential body as JSON.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the HTTP request fails or response JSON cannot be parsed.
    pub async fn auth_set(
        &self,
        provider_id: &str,
        directory: Option<&Path>,
        body: impl Serialize,
    ) -> Result<bool, Error> {
        let url = format!("{}/auth/{}", self.base_url(), provider_id);
        let req = self.http().put(&url).json(&body).with_directory(directory);

        let response = req.send().await?;
        let value: bool = response.json().await?;
        Ok(value)
    }
}
