use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    CapabilityFlag, PromptCapabilities, ResourceCapabilities, ServerTasksCapability,
    ToolCapabilities,
};

/// Capabilities that a server may support.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, CapabilityFlag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<CapabilityFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<ServerTasksCapability>,
}
