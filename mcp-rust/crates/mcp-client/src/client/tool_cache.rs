use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::client::ToolDefinition;

/// Cached tool metadata derived from tools/list.
#[derive(Debug, Default, Clone)]
pub struct ToolCache {
    pub output_schemas: HashMap<String, Value>,
    pub known_task_tools: HashSet<String>,
    pub required_task_tools: HashSet<String>,
}

impl ToolCache {
    pub fn update(&mut self, tools: &[ToolDefinition]) {
        self.output_schemas.clear();
        self.known_task_tools.clear();
        self.required_task_tools.clear();

        for tool in tools {
            if let Some(schema) = tool.output_schema.clone() {
                self.output_schemas.insert(tool.name.clone(), schema);
            }

            let task_support = tool
                .execution
                .as_ref()
                .and_then(|execution| execution.task_support.as_deref());
            match task_support {
                Some("required") => {
                    self.known_task_tools.insert(tool.name.clone());
                    self.required_task_tools.insert(tool.name.clone());
                }
                Some("optional") => {
                    self.known_task_tools.insert(tool.name.clone());
                }
                _ => {}
            }
        }
    }

    pub fn output_schema(&self, tool_name: &str) -> Option<&Value> {
        self.output_schemas.get(tool_name)
    }

    pub fn is_task_tool(&self, tool_name: &str) -> bool {
        self.known_task_tools.contains(tool_name)
    }

    pub fn is_task_required(&self, tool_name: &str) -> bool {
        self.required_task_tools.contains(tool_name)
    }
}
