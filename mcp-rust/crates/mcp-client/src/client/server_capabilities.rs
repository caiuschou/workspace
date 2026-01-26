use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::{CapabilityFlag, PromptCapabilities, ResourceCapabilities, ToolCapabilities};

/// Capabilities advertised by the server.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Value>,
}
