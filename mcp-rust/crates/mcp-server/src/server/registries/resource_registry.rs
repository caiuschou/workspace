use std::collections::HashMap;
use std::sync::Arc;

use mcp_core::types::{Resource, ResourceTemplate};

use crate::server::handlers::ResourceHandler;

/// In-memory registry for resources and resource templates.
#[derive(Default)]
pub struct ResourceRegistry {
    resources: HashMap<String, Resource>,
    handlers: HashMap<String, Arc<dyn ResourceHandler>>,
    templates: HashMap<String, ResourceTemplate>,
}

impl ResourceRegistry {
    pub fn register_resource(&mut self, resource: Resource, handler: impl ResourceHandler) {
        let uri = resource.uri.clone();
        self.resources.insert(uri.clone(), resource);
        self.handlers.insert(uri, Arc::new(handler));
    }

    pub fn register_template(&mut self, template: ResourceTemplate) {
        let name = template.base.name.clone();
        self.templates.insert(name, template);
    }

    pub fn list_resources(&self) -> Vec<Resource> {
        self.resources.values().cloned().collect()
    }

    pub fn list_templates(&self) -> Vec<ResourceTemplate> {
        self.templates.values().cloned().collect()
    }

    pub fn handler(&self, uri: &str) -> Option<Arc<dyn ResourceHandler>> {
        self.handlers.get(uri).cloned()
    }
}
