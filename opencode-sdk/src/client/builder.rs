//! ClientBuilder for configuring the OpenCode HTTP client.
//!
//! Supports request timeout and optional connection pool settings for high-throughput scenarios.

use crate::Error;
use reqwest::Client as ReqwestClient;
use std::time::Duration;

use super::Client;

/// Builder for configuring the OpenCode client.
#[derive(Debug)]
pub struct ClientBuilder {
    pub(super) base_url: String,
    pub(super) timeout: Option<Duration>,
    /// Max idle connections per host (reqwest default: no limit). Set for high concurrency.
    pub(super) pool_max_idle_per_host: Option<usize>,
    /// Idle socket keep-alive timeout (reqwest default: 90s). Pass None to use default.
    pub(super) pool_idle_timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Sets the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the maximum idle connections per host in the connection pool.
    ///
    /// Useful for high-throughput scenarios. Reqwest default is effectively unlimited.
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = Some(max);
        self
    }

    /// Sets how long idle sockets are kept in the pool before being closed.
    ///
    /// Reqwest default is 90 seconds. Pass `None` to use the default.
    pub fn pool_idle_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }

    /// Builds the client. Panics if reqwest client build fails.
    /// Prefer [`try_build`](Self::try_build) when you need to handle errors.
    pub fn build(self) -> Client {
        self.try_build().expect("reqwest client build")
    }

    /// Builds the client, returning an error if reqwest client build fails.
    pub fn try_build(self) -> Result<Client, Error> {
        let mut builder = ReqwestClient::builder();
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(max) = self.pool_max_idle_per_host {
            builder = builder.pool_max_idle_per_host(max);
        }
        if let Some(t) = self.pool_idle_timeout {
            builder = builder.pool_idle_timeout(t);
        }
        let http = builder
            .build()
            .map_err(|e| Error::ClientBuildFailed(e.to_string()))?;
        Ok(Client {
            base_url: self.base_url,
            http,
        })
    }
}
