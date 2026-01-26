use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primitive schema definition for boolean fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BooleanSchema {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}

impl BooleanSchema {
    pub fn new() -> Self {
        Self {
            kind: "boolean".to_string(),
            title: None,
            description: None,
            default: None,
        }
    }
}

impl Default for BooleanSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Primitive schema definition for string fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct StringSchema {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<StringFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// String format validation options.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StringFormat {
    Email,
    Uri,
    Date,
    DateTime,
}

impl StringSchema {
    pub fn new() -> Self {
        Self {
            kind: "string".to_string(),
            title: None,
            description: None,
            min_length: None,
            max_length: None,
            format: None,
            default: None,
        }
    }
}

impl Default for StringSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Primitive schema definition for number fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct NumberSchema {
    #[serde(rename = "type")]
    pub kind: NumberType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NumberType {
    Number,
    Integer,
}

impl NumberSchema {
    pub fn new() -> Self {
        Self {
            kind: NumberType::Number,
            title: None,
            description: None,
            minimum: None,
            maximum: None,
            default: None,
        }
    }

    pub fn integer() -> Self {
        Self {
            kind: NumberType::Integer,
            title: None,
            description: None,
            minimum: None,
            maximum: None,
            default: None,
        }
    }
}

impl Default for NumberSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema for single-selection enumeration without display titles for options.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct UntitledEnumSchema {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

impl UntitledEnumSchema {
    pub fn new(values: Vec<String>) -> Self {
        Self {
            kind: "string".to_string(),
            title: None,
            description: None,
            enum_values: values,
            default: None,
        }
    }
}

/// Option with const value and display title.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EnumOption {
    #[serde(rename = "const")]
    pub const_value: String,
    pub title: String,
}

/// Schema for single-selection enumeration with display titles for each option.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TitledEnumSchema {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "oneOf")]
    pub one_of: Vec<EnumOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

impl TitledEnumSchema {
    pub fn new(options: Vec<EnumOption>) -> Self {
        Self {
            kind: "string".to_string(),
            title: None,
            description: None,
            one_of: options,
            default: None,
        }
    }
}

/// Union of all primitive schema definitions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum PrimitiveSchemaDefinition {
    Boolean(BooleanSchema),
    String(StringSchema),
    Number(NumberSchema),
    UntitledEnum(UntitledEnumSchema),
    TitledEnum(TitledEnumSchema),
}

/// A restricted subset of JSON Schema for elicitation forms.
/// Only top-level properties are allowed, without nesting.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElicitationSchema {
    #[serde(rename = "type")]
    pub kind: String,
    pub properties: HashMap<String, PrimitiveSchemaDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl ElicitationSchema {
    pub fn new() -> Self {
        Self {
            kind: "object".to_string(),
            properties: HashMap::new(),
            required: None,
        }
    }

    pub fn with_property(
        mut self,
        name: impl Into<String>,
        schema: PrimitiveSchemaDefinition,
    ) -> Self {
        self.properties.insert(name.into(), schema);
        self
    }

    pub fn with_required(mut self, fields: Vec<String>) -> Self {
        self.required = Some(fields);
        self
    }
}

impl Default for ElicitationSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Elicitation result content value.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum ElicitationValue {
    String(String),
    Number(f64),
    Boolean(bool),
    StringArray(Vec<String>),
}

impl From<String> for ElicitationValue {
    fn from(s: String) -> Self {
        ElicitationValue::String(s)
    }
}

impl From<&str> for ElicitationValue {
    fn from(s: &str) -> Self {
        ElicitationValue::String(s.to_string())
    }
}

impl From<f64> for ElicitationValue {
    fn from(n: f64) -> Self {
        ElicitationValue::Number(n)
    }
}

impl From<i32> for ElicitationValue {
    fn from(n: i32) -> Self {
        ElicitationValue::Number(n as f64)
    }
}

impl From<bool> for ElicitationValue {
    fn from(b: bool) -> Self {
        ElicitationValue::Boolean(b)
    }
}

impl From<Vec<String>> for ElicitationValue {
    fn from(arr: Vec<String>) -> Self {
        ElicitationValue::StringArray(arr)
    }
}
