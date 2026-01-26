use serde_json::Value;

use crate::client::JsonSchemaValidator;

/// Placeholder validator that accepts all values.
#[derive(Debug, Default, Clone)]
pub struct NoopJsonSchemaValidator;

impl JsonSchemaValidator for NoopJsonSchemaValidator {
    fn validate(&self, _schema: &Value, _data: &Value) -> Result<(), String> {
        Ok(())
    }
}
