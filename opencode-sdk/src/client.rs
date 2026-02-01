//! HTTP client for OpenCode Server API.

use crate::Error;
use reqwest::Client as ReqwestClient;
use std::time::Duration;

/// OpenCode Server API client.
///
/// Provides type-safe access to all OpenCode Server endpoints.
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    http: ReqwestClient,
}

impl Client {
    /// Creates a new client for the given base URL.
    ///
    /// Panics if the underlying reqwest client fails to build (e.g. TLS init).
    /// For fallible construction use [`Client::builder`](Self::builder)(base_url).try_build().
    ///
    /// # Example
    ///
    /// ```
    /// use opencode_sdk::Client;
    ///
    /// let client = Client::new("http://127.0.0.1:4096");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::builder(base_url).build()
    }

    /// Returns a builder for configuring the client.
    pub fn builder(base_url: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            base_url: base_url.into(),
            timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Returns the base URL of the OpenCode Server.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the underlying HTTP client (for internal API modules).
    pub(crate) fn http(&self) -> &ReqwestClient {
        &self.http
    }

    /// Performs a GET request to the health endpoint.
    ///
    /// Use this to verify the server is running and check its version.
    pub async fn health(&self) -> Result<HealthResponse, Error> {
        let url = format!("{}/global/health", self.base_url);
        let response = self.http.get(&url).send().await?;
        let health: crate::HealthResponse = response.json().await?;
        Ok(health)
    }

    /// Disposes all OpenCode instances, releasing all resources.
    ///
    /// `POST /global/dispose`
    pub async fn global_dispose(&self) -> Result<bool, Error> {
        let url = format!("{}/global/dispose", self.base_url);
        let response = self.http.post(&url).send().await?;
        let result: bool = response.json().await?;
        Ok(result)
    }

}

/// Builder for configuring the OpenCode client.
#[derive(Debug)]
pub struct ClientBuilder {
    base_url: String,
    timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Sets the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Builds the client. Panics if reqwest client build fails.
    /// Prefer [`try_build`](Self::try_build) when you need to handle errors.
    pub fn build(self) -> Client {
        self.try_build().expect("reqwest client build")
    }

    /// Builds the client, returning an error if reqwest client build fails.
    pub fn try_build(self) -> Result<Client, crate::Error> {
        let mut builder = ReqwestClient::builder();
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        let http = builder
            .build()
            .map_err(|e| crate::Error::ClientBuildFailed(e.to_string()))?;
        Ok(Client {
            base_url: self.base_url,
            http,
        })
    }
}

/// Response from `/global/health`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HealthResponse {
    /// Server version string.
    pub version: String,
}
