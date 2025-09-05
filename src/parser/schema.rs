use std::collections::HashMap;
use super::TypeRegistry;

/// Parsed schema representation
#[derive(Debug, Default, Clone)]
pub struct ParsedSchema {
    pub protocol: Option<Protocol>,
    pub imports: Vec<String>,
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
    pub typedefs: Vec<TypeDef>,
}

/// Protocol definition
#[derive(Debug, Clone)]
pub struct Protocol {
    pub name: String,
    pub version: String,
    pub namespace: Option<String>,
    pub services: Vec<Service>,
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
}

/// Service definition
#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub methods: Vec<Method>,
    pub streams: Vec<Stream>,
}

/// RPC Method definition
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub request: Option<Message>,
    pub response: Option<Message>,
}

/// Streaming endpoint definition
#[derive(Debug, Clone)]
pub struct Stream {
    pub name: String,
    pub request: Option<Message>,
    pub response: Option<Message>,
}

/// Message/struct definition
#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
}

/// Field definition
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<DefaultValue>,
    pub constraints: Constraints,
}

/// Field type
#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Int,
    Float,
    Bool,
    Json,
    Array(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Enum(Vec<String>),
    Object,
    Custom(String),
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub values: Vec<String>,
}

/// Type definition
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub base_type: String,
    pub rust_type: Option<String>,
    pub typescript_type: Option<String>,
    pub format: Option<String>,
    pub pattern: Option<String>,
}

/// Default value for fields
#[derive(Debug, Clone)]
pub enum DefaultValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<DefaultValue>),
    Object(HashMap<String, DefaultValue>),
    Null,
}

/// Field constraints
#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

impl FieldType {
    /// Get the Rust type representation
    pub fn to_rust_type(&self, type_registry: &TypeRegistry) -> String {
        match self {
            FieldType::String => "String".to_string(),
            FieldType::Int => "i64".to_string(),
            FieldType::Float => "f64".to_string(),
            FieldType::Bool => "bool".to_string(),
            FieldType::Json => "serde_json::Value".to_string(),
            FieldType::Array(inner) => format!("Vec<{}>", inner.to_rust_type(type_registry)),
            FieldType::Map(key, value) => format!(
                "HashMap<{}, {}>",
                key.to_rust_type(type_registry),
                value.to_rust_type(type_registry)
            ),
            FieldType::Enum(_values) => {
                // This should be resolved to the actual enum name
                "String".to_string()
            }
            FieldType::Object => "serde_json::Value".to_string(),
            FieldType::Custom(name) => {
                type_registry.get_rust_type(name)
                    .unwrap_or_else(|| name.clone())
            }
        }
    }

    /// Get the TypeScript type representation
    pub fn to_typescript_type(&self, type_registry: &TypeRegistry) -> String {
        match self {
            FieldType::String => "string".to_string(),
            FieldType::Int | FieldType::Float => "number".to_string(),
            FieldType::Bool => "boolean".to_string(),
            FieldType::Json | FieldType::Object => "any".to_string(),
            FieldType::Array(inner) => format!("{}[]", inner.to_typescript_type(type_registry)),
            FieldType::Map(_, value) => format!(
                "Record<string, {}>",
                value.to_typescript_type(type_registry)
            ),
            FieldType::Enum(values) => {
                values.iter()
                    .map(|v| format!("'{}'", v))
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
            FieldType::Custom(name) => {
                type_registry.get_typescript_type(name)
                    .unwrap_or_else(|| name.clone())
            }
        }
    }
}