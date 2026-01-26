use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ModelHint;

/// The server's preferences for model selection, requested of the client during sampling.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct ModelPreferences {
    /// Optional hints to use for model selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<ModelHint>>,
    /// How much to prioritize cost when selecting a model (0-1).
    #[serde(rename = "costPriority", skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f64>,
    /// How much to prioritize sampling speed (latency) when selecting a model (0-1).
    #[serde(rename = "speedPriority", skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f64>,
    /// How much to prioritize intelligence and capabilities when selecting a model (0-1).
    #[serde(rename = "intelligencePriority", skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f64>,
}
