//! 工具注册表：按名注册与执行。
//!
//! 与 `Tool` trait 配合：`register(Box<dyn Tool>)`、`get(name)`、`execute(name, args)`。
//! `execute` 使用 `validate_args(schema, args)` 做参数校验，再调用 `Tool::execute`。

use std::collections::HashMap;

use crate::error::ToolError;
use crate::tool::Tool;
use crate::tool::validation;
use serde_json::Value;

/// 工具注册表：按名称注册工具，并支持按名执行与参数校验。
///
/// 与 `Tool` 配合使用。`execute(name, args)` 会先解析 schema 中的 `required`，
/// 校验 `args` 是否包含必填字段，再调用 `Tool::execute`。
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// 新建空注册表。
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册一个工具；同名会覆盖。
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// 按名称获取工具（只读访问描述与 schema，不执行）。
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    /// 按名称执行工具；使用 `validate_args(schema, args)` 校验后调用 `Tool::execute`。
    pub fn execute(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        let tool = self.tools.get(name).ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        let schema = tool.parameters_schema();
        validation::validate_args(&schema, &args).map_err(ToolError::from)?;
        tool.execute(args)
    }

    /// 返回已注册工具的名称列表。
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::CalculatorTool;

    #[test]
    fn registry_register_get_execute() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(CalculatorTool::new()));
        assert!(reg.get("calculator").is_some());
        let args = serde_json::json!({"expression": "3+5"});
        let out = reg.execute("calculator", args).unwrap();
        assert_eq!(out, serde_json::json!(8));
    }

    #[test]
    fn registry_not_found() {
        let reg = ToolRegistry::new();
        let args = serde_json::json!({});
        let err = reg.execute("nonexistent", args).unwrap_err();
        assert!(matches!(err, ToolError::NotFound(_)));
    }
}
