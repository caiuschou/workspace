//! TUI (Terminal User Interface) API for OpenCode Server.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use serde::Serialize;
use std::path::Path;

impl Client {
    fn tui_post(
        &self,
        path: &str,
        directory: Option<&Path>,
        body: Option<serde_json::Value>,
    ) -> reqwest::RequestBuilder {
        let url = format!("{}/tui{}", self.base_url(), path);
        let req = match body {
            Some(b) => self.http().post(&url).json(&b),
            None => self.http().post(&url),
        };
        req.with_directory(directory)
    }

    /// Appends text to the TUI prompt.
    ///
    /// `POST /tui/append-prompt`
    pub async fn tui_append_prompt(
        &self,
        directory: Option<&Path>,
        text: &str,
    ) -> Result<bool, Error> {
        let req = self.tui_post("/append-prompt", directory, Some(serde_json::json!({ "text": text })));
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Clears the TUI prompt.
    ///
    /// `POST /tui/clear-prompt`
    pub async fn tui_clear_prompt(&self, directory: Option<&Path>) -> Result<bool, Error> {
        let req = self.tui_post("/clear-prompt", directory, None);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Submits the current TUI prompt.
    ///
    /// `POST /tui/submit-prompt`
    pub async fn tui_submit_prompt(&self, directory: Option<&Path>) -> Result<bool, Error> {
        let req = self.tui_post("/submit-prompt", directory, None);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Opens the help dialog.
    ///
    /// `POST /tui/open-help`
    pub async fn tui_open_help(&self, directory: Option<&Path>) -> Result<bool, Error> {
        let req = self.tui_post("/open-help", directory, None);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Opens the sessions dialog.
    ///
    /// `POST /tui/open-sessions`
    pub async fn tui_open_sessions(&self, directory: Option<&Path>) -> Result<bool, Error> {
        let req = self.tui_post("/open-sessions", directory, None);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Opens the models dialog.
    ///
    /// `POST /tui/open-models`
    pub async fn tui_open_models(&self, directory: Option<&Path>) -> Result<bool, Error> {
        let req = self.tui_post("/open-models", directory, None);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Opens the themes dialog.
    ///
    /// `POST /tui/open-themes`
    pub async fn tui_open_themes(&self, directory: Option<&Path>) -> Result<bool, Error> {
        let req = self.tui_post("/open-themes", directory, None);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Executes a TUI command.
    ///
    /// `POST /tui/execute-command`
    pub async fn tui_execute_command(
        &self,
        directory: Option<&Path>,
        command: &str,
    ) -> Result<bool, Error> {
        let req = self.tui_post("/execute-command", directory, Some(serde_json::json!({ "command": command })));
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Shows a toast notification.
    ///
    /// `POST /tui/show-toast`
    pub async fn tui_show_toast(
        &self,
        directory: Option<&Path>,
        message: &str,
        variant: &str,
        title: Option<&str>,
        duration: Option<u64>,
    ) -> Result<bool, Error> {
        let mut body = serde_json::json!({ "message": message, "variant": variant });
        if let Some(t) = title {
            body["title"] = serde_json::Value::String(t.to_string());
        }
        if let Some(d) = duration {
            body["duration"] = serde_json::Value::Number(serde_json::Number::from(d));
        }
        let req = self.tui_post("/show-toast", directory, Some(body));
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Selects a session in the TUI.
    ///
    /// `POST /tui/select-session`
    pub async fn tui_select_session(
        &self,
        directory: Option<&Path>,
        session_id: &str,
    ) -> Result<bool, Error> {
        let req = self.tui_post("/select-session", directory, Some(serde_json::json!({ "sessionID": session_id })));
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Publishes a TUI event.
    ///
    /// `POST /tui/publish`
    pub async fn tui_publish(
        &self,
        directory: Option<&Path>,
        body: impl Serialize,
    ) -> Result<bool, Error> {
        let req = self.tui_post("/publish", directory, Some(serde_json::to_value(body)?));
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Gets the next TUI control request.
    ///
    /// `GET /tui/control/next`
    pub async fn tui_control_next(
        &self,
        directory: Option<&Path>,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/tui/control/next", self.base_url());
        let req = self.http().get(&url).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }

    /// Submits a response to the TUI control request queue.
    ///
    /// `POST /tui/control/response`
    pub async fn tui_control_response(
        &self,
        directory: Option<&Path>,
        body: impl Serialize,
    ) -> Result<bool, Error> {
        let url = format!("{}/tui/control/response", self.base_url());
        let req = self.http().post(&url).json(&body).with_directory(directory);
        let response = req.send().await?;
        Ok(response.json().await?)
    }
}
