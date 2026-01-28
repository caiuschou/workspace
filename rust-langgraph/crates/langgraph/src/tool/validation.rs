//! Argument validation against JSON Schema (minimal).
//!
//! - `validate_args(schema, args)`: checks that `args` contains all required fields from `schema`.
//! - Used by `ToolRegistry::execute`; errors are converted to `ToolError::ValidationFailed` via `ValidationError`.

use crate::error::ValidationError;
use serde_json::Value;

/// Validates `args` against the minimal schema: ensures `args` is an object and
/// contains every key listed in `schema["required"]`.
///
/// Used by `ToolRegistry::execute` before calling `Tool::execute`. Returns `Ok(())` when
/// schema has no `required` or when all required keys are present; returns `Err(ValidationError)`
/// when `args` is not an object or when a required key is missing.
pub fn validate_args(schema: &Value, args: &Value) -> Result<(), ValidationError> {
    let Some(req) = schema.get("required") else {
        return Ok(());
    };
    let Some(arr) = req.as_array() else {
        return Ok(());
    };
    let Some(obj) = args.as_object() else {
        return Err(ValidationError("args must be an object".into()));
    };
    for r in arr {
        let Some(s) = r.as_str() else {
            continue;
        };
        if !obj.contains_key(s) {
            return Err(ValidationError(format!("missing required field: {}", s)));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_args_no_required() {
        let schema = serde_json::json!({"type": "object"});
        let args = serde_json::json!({});
        assert!(validate_args(&schema, &args).is_ok());
    }

    #[test]
    fn validate_args_required_ok() {
        let schema = serde_json::json!({"required": ["a", "b"]});
        let args = serde_json::json!({"a": 1, "b": 2});
        assert!(validate_args(&schema, &args).is_ok());
    }

    #[test]
    fn validate_args_missing_field() {
        let schema = serde_json::json!({"required": ["a", "b"]});
        let args = serde_json::json!({"a": 1});
        let e = validate_args(&schema, &args).unwrap_err();
        assert!(e.0.contains("b"));
    }

    #[test]
    fn validate_args_not_object() {
        let schema = serde_json::json!({"required": ["a"]});
        let args = serde_json::json!([]);
        let e = validate_args(&schema, &args).unwrap_err();
        assert!(e.0.contains("object"));
    }
}
