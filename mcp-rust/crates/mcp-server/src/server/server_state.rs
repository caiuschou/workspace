use std::collections::HashMap;

use mcp_core::types::{ClientCapabilities, Implementation, LoggingLevel, ServerCapabilities};

/// Mutable server state shared with handlers.
#[derive(Debug, Clone)]
pub struct ServerState {
    pub capabilities: ServerCapabilities,
    pub instructions: Option<String>,
    pub client_capabilities: Option<ClientCapabilities>,
    pub client_info: Option<Implementation>,
    pub capabilities_locked: bool,
    pub logging_levels: HashMap<Option<String>, LoggingLevel>,
}

impl ServerState {
    pub fn new(capabilities: ServerCapabilities, instructions: Option<String>) -> Self {
        Self {
            capabilities,
            instructions,
            client_capabilities: None,
            client_info: None,
            capabilities_locked: false,
            logging_levels: HashMap::new(),
        }
    }
}
