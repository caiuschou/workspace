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

    /// Performs a GET request to the health endpoint.
    ///
    /// Use this to verify the server is running and check its version.
    pub async fn health(&self) -> Result<HealthResponse, Error> {
        let url = format!("{}/global/health", self.base_url);
        let response = self.http.get(&url).send().await?;
        let health: crate::HealthResponse = response.json().await?;
        Ok(health)
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

    /// Builds the client.
    pub fn build(self) -> Client {
        let mut builder = ReqwestClient::builder();
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        let http = builder.build().expect("reqwest client build");
        Client {
            base_url: self.base_url,
            http,
        }
    }
}

/// Response from `/global/health`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HealthResponse {
    /// Server version string.
    pub version: String,
}
