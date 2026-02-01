//! Provider API for OpenCode Server.
//!
//! List providers and manage OAuth authentication.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

/// Request body for `POST /provider/{providerID}/oauth/authorize`.
#[derive(Debug, Clone, Serialize)]
pub struct OAuthAuthorizeRequest {
    /// Auth method index.
    pub method: u32,
}

/// Request body for `POST /provider/{providerID}/oauth/callback`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthCallbackRequest {
    /// Auth method index.
    pub method: u32,
    /// OAuth authorization code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl Client {
    /// Lists all available AI providers.
    ///
    /// `GET /provider`
    pub async fn provider_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/provider", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Gets authentication methods for all AI providers.
    ///
    /// `GET /provider/auth`
    pub async fn provider_auth(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/provider/auth", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Initiates OAuth authorization for a provider.
    ///
    /// `POST /provider/{providerID}/oauth/authorize`
    pub async fn provider_oauth_authorize(
        &self,
        provider_id: &str,
        directory: Option<&Path>,
        request: OAuthAuthorizeRequest,
    ) -> Result<serde_json::Value, Error> {
        let url = format!(
            "{}/provider/{}/oauth/authorize",
            self.base_url(),
            provider_id
        );
        let req = self.http().post(&url).json(&request).with_directory(directory);

        let response = req.send().await?;
        let value: serde_json::Value = response.json().await?;
        Ok(value)
    }

    /// Handles OAuth callback from a provider.
    ///
    /// `POST /provider/{providerID}/oauth/callback`
    pub async fn provider_oauth_callback(
        &self,
        provider_id: &str,
        directory: Option<&Path>,
        request: OAuthCallbackRequest,
    ) -> Result<bool, Error> {
        let url = format!(
            "{}/provider/{}/oauth/callback",
            self.base_url(),
            provider_id
        );
        let req = self.http().post(&url).json(&request).with_directory(directory);

        let response = req.send().await?;
        let value: bool = response.json().await?;
        Ok(value)
    }
}
