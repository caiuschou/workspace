//! Shared types for Agent registration, discovery and messaging.
//!
//! See docs/registration-discovery.md and docs/agent-communication.md for protocol details.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Agent record stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    pub registered_at: DateTime<Utc>,
}
