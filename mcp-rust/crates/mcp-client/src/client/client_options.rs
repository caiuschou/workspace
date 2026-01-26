use std::sync::Arc;

use crate::client::{
    ClientCapabilities, Implementation, JsonSchemaValidator, ListChangedHandlers,
    NoopJsonSchemaValidator,
};

/// Options provided when constructing a client.
#[derive(Clone)]
pub struct ClientOptions {
    pub client_info: Implementation,
    pub protocol_version: String,
    pub capabilities: Option<ClientCapabilities>,
    pub list_changed: Option<ListChangedHandlers>,
    pub json_schema_validator: Option<Arc<dyn JsonSchemaValidator>>,
    pub roots: Option<Vec<mcp_core::types::Root>>,
}

impl ClientOptions {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            client_info: Implementation::new(name),
            protocol_version: "0.1.0".to_string(),
            capabilities: None,
            list_changed: None,
            json_schema_validator: Some(Arc::new(NoopJsonSchemaValidator::default())),
            roots: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.client_info = self.client_info.with_version(version);
        self
    }

    pub fn with_protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = version.into();
        self
    }

    pub fn with_capabilities(mut self, capabilities: ClientCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    pub fn with_list_changed(mut self, list_changed: ListChangedHandlers) -> Self {
        self.list_changed = Some(list_changed);
        self
    }

    pub fn with_json_schema_validator(mut self, validator: Arc<dyn JsonSchemaValidator>) -> Self {
        self.json_schema_validator = Some(validator);
        self
    }

    pub fn with_roots(mut self, roots: Vec<mcp_core::types::Root>) -> Self {
        self.roots = Some(roots);
        self
    }
}
