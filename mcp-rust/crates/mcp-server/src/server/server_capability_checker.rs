use std::sync::{Arc, Mutex};

use mcp_core::protocol::{CapabilityChecker, ProtocolError};
use mcp_core::types::ServerCapabilities;

use super::server_state::ServerState;

/// Capability checker tied to server state.
pub struct ServerCapabilityChecker {
    state: Arc<Mutex<ServerState>>,
}

impl ServerCapabilityChecker {
    pub fn new(state: Arc<Mutex<ServerState>>) -> Self {
        Self { state }
    }

    fn capabilities(&self) -> ServerCapabilities {
        self.state
            .lock()
            .expect("server state")
            .capabilities
            .clone()
    }
}

impl CapabilityChecker for ServerCapabilityChecker {
    fn assert_request(&self, method: &str) -> Result<(), ProtocolError> {
        let capabilities = self.capabilities();
        match method {
            "logging/setLevel" => {
                if capabilities.logging.is_none() {
                    return Err(ProtocolError::Capability(
                        "logging capability not enabled".to_string(),
                    ));
                }
            }
            "prompts/get" | "prompts/list" => {
                if capabilities.prompts.is_none() {
                    return Err(ProtocolError::Capability(
                        "prompts capability not enabled".to_string(),
                    ));
                }
            }
            "resources/list" | "resources/templates/list" | "resources/read" => {
                if capabilities.resources.is_none() {
                    return Err(ProtocolError::Capability(
                        "resources capability not enabled".to_string(),
                    ));
                }
            }
            "tools/list" | "tools/call" => {
                if capabilities.tools.is_none() {
                    return Err(ProtocolError::Capability(
                        "tools capability not enabled".to_string(),
                    ));
                }
            }
            "tasks/get" | "tasks/list" | "tasks/result" | "tasks/cancel" => {
                if capabilities.tasks.is_none() {
                    return Err(ProtocolError::Capability(
                        "tasks capability not enabled".to_string(),
                    ));
                }
            }
            "initialize" | "ping" => {}
            _ => {}
        }
        Ok(())
    }

    fn assert_notification(&self, _method: &str) -> Result<(), ProtocolError> {
        Ok(())
    }

    fn assert_request_handler(&self, method: &str) -> Result<(), ProtocolError> {
        self.assert_request(method)
    }

    fn assert_notification_handler(&self, _method: &str) -> Result<(), ProtocolError> {
        Ok(())
    }
}
