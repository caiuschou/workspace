use jsonschema::{Draft, ValidationOptions};
use schemars::JsonSchema;
use schemars::schema::RootSchema;
use serde_json::{Value, to_value};
use thiserror::Error;

/// Marker trait for schema validators so protocols can stay generic.
pub trait SchemaValidator: Send + Sync + 'static {
    type Schema: Clone + Send + Sync;

    /// Validate the provided payload against the schema.
    fn validate(&self, schema: &Self::Schema, payload: &Value) -> Result<(), ValidationError>;
}

/// Default validator that builds on `schemars` + `jsonschema`.
#[derive(Debug, Clone)]
pub struct JsonSchemaValidator {
    draft: Draft,
}

impl JsonSchemaValidator {
    /// Create a validator that targets `Draft 2020-12`.
    pub fn new() -> Self {
        Self {
            draft: Draft::Draft202012,
        }
    }

    /// Allow overriding the draft used for compilation.
    pub fn with_draft(draft: Draft) -> Self {
        Self { draft }
    }
}

impl Default for JsonSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaValidator for JsonSchemaValidator {
    type Schema = RootSchema;

    fn validate(&self, schema: &RootSchema, payload: &Value) -> Result<(), ValidationError> {
        let schema_value =
            to_value(schema).map_err(|err| ValidationError::Schema(err.to_string()))?;

        let compiled = ValidationOptions::default()
            .with_draft(self.draft)
            .build(&schema_value)
            .map_err(|err| ValidationError::Schema(err.to_string()))?;

        compiled
            .validate(payload)
            .map_err(|errors| ValidationError::Failed(errors.map(|e| e.to_string()).collect()))
    }
}

impl JsonSchemaValidator {
    /// Convenience helper so callers can infer schemas from Rust types.
    pub fn schema_for<T: JsonSchema>() -> RootSchema {
        schemars::schema_for!(T)
    }
}

/// Errors emitted during schema validation.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("failed to compile schema: {0}")]
    Schema(String),

    #[error("validation failed: {0:?}")]
    Failed(Vec<String>),
}
