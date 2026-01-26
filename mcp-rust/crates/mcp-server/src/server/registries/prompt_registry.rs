use std::collections::HashMap;
use std::sync::Arc;

use mcp_core::types::Prompt;

use crate::server::handlers::PromptHandler;

/// In-memory registry for prompts.
#[derive(Default)]
pub struct PromptRegistry {
    prompts: HashMap<String, Prompt>,
    handlers: HashMap<String, Arc<dyn PromptHandler>>,
}

impl PromptRegistry {
    pub fn register_prompt(&mut self, prompt: Prompt, handler: impl PromptHandler) {
        let name = prompt.base.name.clone();
        self.prompts.insert(name.clone(), prompt);
        self.handlers.insert(name, Arc::new(handler));
    }

    pub fn list_prompts(&self) -> Vec<Prompt> {
        self.prompts.values().cloned().collect()
    }

    pub fn prompt(&self, name: &str) -> Option<Prompt> {
        self.prompts.get(name).cloned()
    }

    pub fn handler(&self, name: &str) -> Option<Arc<dyn PromptHandler>> {
        self.handlers.get(name).cloned()
    }
}
