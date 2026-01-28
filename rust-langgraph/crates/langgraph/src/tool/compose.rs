//! 工具组合：将多个工具按序串联执行。
//!
//! - `ToolChain`: 实现 `Tool`，将上一工具输出作为下一工具的 `input` 参数传入
//! - 与 `ToolRegistry` 配合，按给定工具名顺序执行

use std::sync::Arc;

use crate::error::ToolError;
use crate::tool::{Tool, ToolRegistry};
use serde_json::Value;

/// 工具链：按顺序执行多个工具，上一工具的输出作为下一工具的 `input` 参数。
///
/// Schema 与第一个工具一致；execute 时依次执行，每次将上一结果放入 `{"input": prev}` 传给下一工具。
/// Interacts with `ToolRegistry::execute` via `Tool::execute`.
pub struct ToolChain {
    /// 注册表，用于查找并执行各工具。
    registry: Arc<ToolRegistry>,
    /// 按序执行的工具名。
    tool_names: Vec<String>,
}

impl ToolChain {
    /// 新建工具链，按 `tool_names` 顺序执行。
    pub fn new(registry: Arc<ToolRegistry>, tool_names: Vec<String>) -> Self {
        Self {
            registry,
            tool_names,
        }
    }

    /// 第一个工具的名称，用于取 schema。
    fn first_tool(&self) -> Result<&dyn Tool, ToolError> {
        let name = self
            .tool_names
            .first()
            .ok_or_else(|| ToolError::ValidationFailed("empty tool chain".into()))?;
        self.registry
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))
    }
}

impl Tool for ToolChain {
    fn name(&self) -> &str {
        "chain"
    }

    fn description(&self) -> &str {
        "Runs a sequence of tools; each tool receives the previous output as its 'input' argument."
    }

    fn parameters_schema(&self) -> Value {
        self.first_tool()
            .map(|t| t.parameters_schema())
            .unwrap_or_else(|_| {
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                })
            })
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let mut current = args;
        for (i, name) in self.tool_names.iter().enumerate() {
            current = self.registry.execute(name, current)?;
            if i + 1 < self.tool_names.len() {
                current = serde_json::json!({ "input": current });
            }
        }
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::CalculatorTool;

    #[test]
    fn tool_chain_single_tool() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(CalculatorTool::new()));
        let registry = Arc::new(reg);
        let chain = ToolChain::new(registry, vec!["calculator".into()]);
        let args = serde_json::json!({"expression": "1+2"});
        let out = chain.execute(args).unwrap();
        assert_eq!(out, serde_json::json!(3));
    }
}
