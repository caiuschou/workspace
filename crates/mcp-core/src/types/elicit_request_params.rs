use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ElicitationSchema, TaskMetadata};

/// Elicitation mode.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationMode {
    Form,
    Url,
}

impl Default for ElicitationMode {
    fn default() -> Self {
        Self::Form
    }
}

/// Parameters for an `elicitation/create` request for form-based elicitation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElicitRequestFormParams {
    /// The elicitation mode.
    /// Optional for backward compatibility. Clients MUST treat missing mode as "form".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ElicitationMode>,
    /// The message to present to the user describing what information is being requested.
    pub message: String,
    /// A restricted subset of JSON Schema for the form fields.
    #[serde(rename = "requestedSchema")]
    pub requested_schema: ElicitationSchema,
    /// If specified, request task-augmented execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskMetadata>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

impl ElicitRequestFormParams {
    pub fn new(message: impl Into<String>, schema: ElicitationSchema) -> Self {
        Self {
            mode: Some(ElicitationMode::Form),
            message: message.into(),
            requested_schema: schema,
            task: None,
            meta: None,
        }
    }
}

/// Parameters for an `elicitation/create` request for URL-based elicitation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElicitRequestUrlParams {
    /// The elicitation mode.
    pub mode: ElicitationMode,
    /// The message to present to the user explaining why the interaction is needed.
    pub message: String,
    /// The ID of the elicitation, which must be unique within the context of the server.
    #[serde(rename = "elicitationId")]
    pub elicitation_id: String,
    /// The URL that the user should navigate to.
    pub url: String,
    /// If specified, request task-augmented execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskMetadata>,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

impl ElicitRequestUrlParams {
    pub fn new(
        message: impl Into<String>,
        elicitation_id: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            mode: ElicitationMode::Url,
            message: message.into(),
            elicitation_id: elicitation_id.into(),
            url: url.into(),
            task: None,
            meta: None,
        }
    }
}

/// The parameters for a request to elicit additional information from the user via the client.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum ElicitRequestParams {
    Form(ElicitRequestFormParams),
    Url(ElicitRequestUrlParams),
}

impl From<ElicitRequestFormParams> for ElicitRequestParams {
    fn from(params: ElicitRequestFormParams) -> Self {
        ElicitRequestParams::Form(params)
    }
}

impl From<ElicitRequestUrlParams> for ElicitRequestParams {
    fn from(params: ElicitRequestUrlParams) -> Self {
        ElicitRequestParams::Url(params)
    }
}
