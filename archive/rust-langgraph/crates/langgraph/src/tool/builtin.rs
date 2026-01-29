//! 内置工具：Calculator 等。
//!
//! - `CalculatorTool`: 安全算术表达式求值，参数 `expression: string`

use crate::error::ToolError;
use crate::tool::Tool;
use evalexpr::eval;
use serde_json::Value;

/// 计算器工具：对算术表达式字符串做安全求值。
///
/// 使用 `evalexpr` 求值，支持加减乘除与常见数学表达式。
/// 与 `ToolRegistry` 配合：注册后 ReAct 可调用 `calculator`，参数 `{"expression": "3+5"}`。
#[derive(Debug, Default)]
pub struct CalculatorTool;

impl CalculatorTool {
    /// 新建计算器工具。
    pub fn new() -> Self {
        Self
    }
}

impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluates a mathematical expression and returns the result. Example: 3+5, 2*10."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": { "type": "string", "description": "Arithmetic expression to evaluate, e.g. 3+5" }
            },
            "required": ["expression"]
        })
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let expr = args
            .get("expression")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::ValidationFailed("missing or non-string 'expression'".into()))?;
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(ToolError::ValidationFailed("expression must be non-empty".into()));
        }
        let result = eval(expr).map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
        let out: Value = if let Ok(i) = result.as_int() {
            serde_json::json!(i)
        } else if let Ok(f) = result.as_float() {
            serde_json::json!(f)
        } else {
            serde_json::json!(result.to_string())
        };
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_3_plus_5() {
        let t = CalculatorTool::new();
        let args = serde_json::json!({"expression": "3+5"});
        let out = t.execute(args).unwrap();
        assert_eq!(out, serde_json::json!(8));
    }

    #[test]
    fn calculator_float() {
        let t = CalculatorTool::new();
        let args = serde_json::json!({"expression": "1.0 + 2 * 3"});
        let out = t.execute(args).unwrap();
        assert_eq!(out, serde_json::json!(7.0));
    }

    #[test]
    fn calculator_invalid_expression() {
        let t = CalculatorTool::new();
        let args = serde_json::json!({"expression": "1 + "});
        let err = t.execute(args).unwrap_err();
        assert!(matches!(err, ToolError::ExecutionFailed(_)));
    }

    #[test]
    fn calculator_missing_expression() {
        let t = CalculatorTool::new();
        let args = serde_json::json!({});
        let err = t.execute(args).unwrap_err();
        assert!(matches!(err, ToolError::ValidationFailed(_)));
    }
}
