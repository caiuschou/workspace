use std::collections::HashMap;
use std::sync::Arc;

use mcp_core::types::Tool;

use crate::server::handlers::ToolHandler;

/// In-memory registry for tools.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
}

impl ToolRegistry {
    pub fn register_tool(&mut self, tool: Tool, handler: impl ToolHandler) {
        let name = tool.base.name.clone();
        self.tools.insert(name.clone(), tool);
        self.handlers.insert(name, Arc::new(handler));
    }

    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    pub fn tool(&self, name: &str) -> Option<Tool> {
        self.tools.get(name).cloned()
    }

    pub fn handler(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.handlers.get(name).cloned()
    }
}
