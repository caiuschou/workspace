use serde_json::Value;

/// Validates tool output against a JSON schema.
pub trait JsonSchemaValidator: Send + Sync {
    fn validate(&self, schema: &Value, data: &Value) -> Result<(), String>;
}
