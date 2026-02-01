//! Question API for OpenCode Server.
//!
//! List and respond to question requests.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

impl Client {
    /// Lists all pending question requests.
    ///
    /// `GET /question`
    pub async fn question_list(
        &self,
        directory: Option<&Path>,
    ) -> Result<Vec<serde_json::Value>, Error> {
        let url = format!("{}/question", self.base_url());
        let req = self.http().get(&url).with_directory(directory);

        let response = req.send().await?;
        let body = response.text().await?;
        let value: Vec<serde_json::Value> =
            serde_json::from_str(&body).unwrap_or_default();
        Ok(value)
    }

    /// Replies to a question request.
    ///
    /// `POST /question/{requestID}/reply`. Body: `{ "answers": [[...], ...] }`
    pub async fn question_reply(
        &self,
        request_id: &str,
        directory: Option<&Path>,
        body: impl Serialize,
    ) -> Result<bool, Error> {
        let url = format!("{}/question/{}/reply", self.base_url(), request_id);
        let req = self.http().post(&url).json(&body).with_directory(directory);

        let response = req.send().await?;
        let value: bool = response.json().await?;
        Ok(value)
    }

    /// Rejects a question request.
    ///
    /// `POST /question/{requestID}/reject`
    pub async fn question_reject(
        &self,
        request_id: &str,
        directory: Option<&Path>,
    ) -> Result<bool, Error> {
        let url = format!("{}/question/{}/reject", self.base_url(), request_id);
        let req = self.http().post(&url).with_directory(directory);

        let response = req.send().await?;
        let value: bool = response.json().await?;
        Ok(value)
    }
}
