//! HTTP client for Zoekt JSON API.
//!
//! Calls `POST {base}/api/search` and `POST {base}/api/list` as per
//! zoekt-webserver RPC. See docs/zoekt/json-api.md for schema.
//! Supports HTTP Basic auth via ZOEKT_USERNAME / ZOEKT_PASSWORD when both are set.

use crate::config::{zoekt_base_url, zoekt_basic_auth};
use reqwest::Client;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZoektClientError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Zoekt API error: {0}")]
    Api(String),
}

/// Zoekt JSON API client. Base URL from ZOEKT_BASE_URL or default.
/// Optional HTTP Basic auth when ZOEKT_USERNAME and ZOEKT_PASSWORD are both set.
#[derive(Clone)]
pub struct ZoektClient {
    client: Client,
    base_url: String,
    basic_auth: Option<(String, String)>,
}

impl ZoektClient {
    /// Build client from env: ZOEKT_BASE_URL, and optionally ZOEKT_USERNAME + ZOEKT_PASSWORD for Basic auth.
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            base_url: zoekt_base_url(),
            basic_auth: zoekt_basic_auth(),
        }
    }

    /// Build client with explicit base URL and optional Basic auth (username, password).
    #[allow(dead_code)]
    pub fn new(base_url: String, basic_auth: Option<(String, String)>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            basic_auth,
        }
    }

    fn apply_basic_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some((user, pass)) = &self.basic_auth {
            req.basic_auth(user, Some(pass))
        } else {
            req
        }
    }

    /// POST /api/search. Body: { "Q", "RepoIDs"?, "Opts"? }.
    /// Returns raw JSON object under "Result" key.
    pub async fn search(
        &self,
        q: &str,
        repo_ids: Option<Vec<u32>>,
        opts: Option<Value>,
    ) -> Result<Value, ZoektClientError> {
        let mut body = json!({ "Q": q });
        if let Some(ids) = repo_ids {
            body["RepoIDs"] = serde_json::to_value(ids).unwrap_or(json!([]));
        }
        if let Some(o) = opts {
            body["Opts"] = o;
        }

        let url = format!("{}/api/search", self.base_url);
        let req = self.client.post(&url).json(&body);
        let resp = self.apply_basic_auth(req).send().await?;

        let status = resp.status();
        let bytes = resp.bytes().await?;
        let v: Value = serde_json::from_slice(&bytes).unwrap_or(json!(null));

        if !status.is_success() {
            let msg = v.get("Error").and_then(|e| e.as_str()).unwrap_or("unknown error");
            return Err(ZoektClientError::Api(msg.to_string()));
        }

        Ok(v.get("Result").cloned().unwrap_or(json!(null)))
    }

    /// POST /api/list. Body: { "Q", "Opts"? }.
    /// Returns raw JSON object under "List" key.
    pub async fn list(&self, q: &str, opts: Option<Value>) -> Result<Value, ZoektClientError> {
        let body = if let Some(o) = opts {
            json!({ "Q": q, "Opts": o })
        } else {
            json!({ "Q": q })
        };

        let url = format!("{}/api/list", self.base_url);
        let req = self.client.post(&url).json(&body);
        let resp = self.apply_basic_auth(req).send().await?;

        let status = resp.status();
        let bytes = resp.bytes().await?;
        let v: Value = serde_json::from_slice(&bytes).unwrap_or(json!(null));

        if !status.is_success() {
            let msg = v.get("Error").and_then(|e| e.as_str()).unwrap_or("unknown error");
            return Err(ZoektClientError::Api(msg.to_string()));
        }

        Ok(v.get("List").cloned().unwrap_or(json!(null)))
    }
}
