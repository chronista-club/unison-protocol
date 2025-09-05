use anyhow::{Context, Result};
use kdl::{KdlDocument, KdlNode, KdlEntry, KdlValue};
use indexmap::IndexMap;
use std::collections::HashMap;
use thiserror::Error;

pub mod schema;
pub mod types;

pub use schema::*;
pub use types::*;

/// Parser errors for Unison Protocol
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("KDL parsing error: {0}")]
    Kdl(#[from] kdl::KdlError),
    #[error("Schema validation error: {0}")]
    Validation(String),
    #[error("Type error: {0}")]
    Type(String),
    #[error("Generic parsing error: {0}")]
    Generic(String),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

/// Main schema parser for KDL protocol definitions
pub struct SchemaParser {
    type_registry: TypeRegistry,
}

impl SchemaParser {
    pub fn new() -> Self {
        Self {
            type_registry: TypeRegistry::new(),
        }
    }

    /// Parse a KDL schema string into a ParsedSchema
    pub fn parse(&self, input: &str) -> Result<ParsedSchema> {
        let doc: KdlDocument = input.parse()
            .context("Failed to parse KDL document")?;

        let mut schema = ParsedSchema::default();
        
        for node in doc.nodes() {
            match node.name().value() {
                "protocol" => {
                    schema.protocol = Some(self.parse_protocol(node)?);
                }
                "types" => {
                    self.parse_types(node, &mut schema)?;
                }
                "import" => {
                    if let Some(path) = node.get(0).and_then(|e| e.value().as_string()) {
                        schema.imports.push(path.to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(schema)
    }

    fn parse_protocol(&self, node: &KdlNode) -> Result<Protocol> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("Protocol name is required")?
            .to_string();

        let version = node.get("version")
            .and_then(|e| e.value().as_string())
            .unwrap_or("1.0.0")
            .to_string();

        let mut protocol = Protocol {
            name,
            version,
            namespace: None,
            services: Vec::new(),
            messages: Vec::new(),
            enums: Vec::new(),
        };

        if let Some(children) = node.children() {
            for child in children.nodes() {
                match child.name().value() {
                    "namespace" => {
                        protocol.namespace = child.get(0)
                            .and_then(|e| e.value().as_string())
                            .map(String::from);
                    }
                    "service" => {
                        protocol.services.push(self.parse_service(child)?);
                    }
                    "message" => {
                        protocol.messages.push(self.parse_message(child)?);
                    }
                    "enum" => {
                        protocol.enums.push(self.parse_enum(child)?);
                    }
                    "import" => {
                        // Handle imports within protocol
                    }
                    _ => {}
                }
            }
        }

        Ok(protocol)
    }

    fn parse_service(&self, node: &KdlNode) -> Result<Service> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("Service name is required")?
            .to_string();

        let mut service = Service {
            name,
            methods: Vec::new(),
            streams: Vec::new(),
        };

        if let Some(children) = node.children() {
            for child in children.nodes() {
                match child.name().value() {
                    "method" => {
                        service.methods.push(self.parse_method(child)?);
                    }
                    "stream" => {
                        service.streams.push(self.parse_stream(child)?);
                    }
                    _ => {}
                }
            }
        }

        Ok(service)
    }

    fn parse_method(&self, node: &KdlNode) -> Result<Method> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("Method name is required")?
            .to_string();

        let mut method = Method {
            name,
            request: None,
            response: None,
        };

        if let Some(children) = node.children() {
            for child in children.nodes() {
                match child.name().value() {
                    "request" => {
                        method.request = Some(self.parse_message_inline(child)?);
                    }
                    "response" => {
                        method.response = Some(self.parse_message_inline(child)?);
                    }
                    _ => {}
                }
            }
        }

        Ok(method)
    }

    fn parse_stream(&self, node: &KdlNode) -> Result<Stream> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("Stream name is required")?
            .to_string();

        let mut stream = Stream {
            name,
            request: None,
            response: None,
        };

        if let Some(children) = node.children() {
            for child in children.nodes() {
                match child.name().value() {
                    "request" => {
                        stream.request = Some(self.parse_message_inline(child)?);
                    }
                    "response" => {
                        stream.response = Some(self.parse_message_inline(child)?);
                    }
                    _ => {}
                }
            }
        }

        Ok(stream)
    }

    fn parse_message(&self, node: &KdlNode) -> Result<Message> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("Message name is required")?
            .to_string();

        self.parse_message_with_name(node, name)
    }

    fn parse_message_inline(&self, node: &KdlNode) -> Result<Message> {
        let name = format!("_inline_{}", uuid::Uuid::new_v4());
        self.parse_message_with_name(node, name)
    }

    fn parse_message_with_name(&self, node: &KdlNode, name: String) -> Result<Message> {
        let mut message = Message {
            name,
            fields: Vec::new(),
        };

        if let Some(children) = node.children() {
            for child in children.nodes() {
                if child.name().value() == "field" {
                    message.fields.push(self.parse_field(child)?);
                }
            }
        }

        Ok(message)
    }

    fn parse_field(&self, node: &KdlNode) -> Result<Field> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("Field name is required")?
            .to_string();

        let field_type = node.get("type")
            .and_then(|e| e.value().as_string())
            .context("Field type is required")?;

        let mut field = Field {
            name,
            field_type: self.parse_field_type(field_type, node)?,
            required: node.get("required")
                .and_then(|e| e.value().as_bool())
                .unwrap_or(false),
            default: self.parse_default_value(node),
            constraints: self.parse_constraints(node),
        };

        Ok(field)
    }

    fn parse_field_type(&self, type_str: &str, node: &KdlNode) -> Result<FieldType> {
        Ok(match type_str {
            "string" => FieldType::String,
            "int" => FieldType::Int,
            "float" => FieldType::Float,
            "bool" => FieldType::Bool,
            "json" => FieldType::Json,
            "array" => {
                let item_type = node.get("item_type")
                    .and_then(|e| e.value().as_string())
                    .context("Array item_type is required")?;
                FieldType::Array(Box::new(self.parse_field_type(item_type, node)?))
            }
            "enum" => {
                let values = self.parse_enum_values(node)?;
                FieldType::Enum(values)
            }
            "object" => FieldType::Object,
            _ => FieldType::Custom(type_str.to_string()),
        })
    }

    fn parse_enum(&self, node: &KdlNode) -> Result<Enum> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("Enum name is required")?
            .to_string();

        let mut enum_def = Enum {
            name,
            values: Vec::new(),
        };

        if let Some(children) = node.children() {
            for child in children.nodes() {
                if child.name().value() == "values" {
                    enum_def.values = self.parse_enum_values(child)?;
                }
            }
        }

        Ok(enum_def)
    }

    fn parse_enum_values(&self, node: &KdlNode) -> Result<Vec<String>> {
        // Try to parse from entries first
        let mut values = Vec::new();
        for entry in node.entries() {
            if let Some(s) = entry.value().as_string() {
                values.push(s.to_string());
            }
        }
        
        Ok(values)
    }

    fn parse_default_value(&self, node: &KdlNode) -> Option<DefaultValue> {
        node.get("default").map(|e| {
            match e.value() {
                KdlValue::String(s) => DefaultValue::String(s.to_string()),
                KdlValue::Base10(i) => DefaultValue::Int(*i),
                KdlValue::Base10Float(f) => DefaultValue::Float(*f),
                KdlValue::Bool(b) => DefaultValue::Bool(*b),
                _ => DefaultValue::Null,
            }
        })
    }

    fn parse_constraints(&self, node: &KdlNode) -> Constraints {
        Constraints {
            min: node.get("min").and_then(|e| e.value().as_i64()),
            max: node.get("max").and_then(|e| e.value().as_i64()),
            min_length: node.get("min_length").and_then(|e| e.value().as_i64()).map(|i| i as usize),
            max_length: node.get("max_length").and_then(|e| e.value().as_i64()).map(|i| i as usize),
            pattern: node.get("pattern").and_then(|e| e.value().as_string()).map(String::from),
        }
    }

    fn parse_types(&self, node: &KdlNode, schema: &mut ParsedSchema) -> Result<()> {
        if let Some(children) = node.children() {
            for child in children.nodes() {
                match child.name().value() {
                    "typedef" => {
                        schema.typedefs.push(self.parse_typedef(child)?);
                    }
                    "message" => {
                        schema.messages.push(self.parse_message(child)?);
                    }
                    "enum" => {
                        schema.enums.push(self.parse_enum(child)?);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn parse_typedef(&self, node: &KdlNode) -> Result<TypeDef> {
        let name = node.get(0)
            .and_then(|e| e.value().as_string())
            .context("TypeDef name is required")?
            .to_string();

        let mut typedef = TypeDef {
            name,
            base_type: String::new(),
            rust_type: None,
            typescript_type: None,
            format: None,
            pattern: None,
        };

        if let Some(children) = node.children() {
            for child in children.nodes() {
                match child.name().value() {
                    "base_type" => {
                        typedef.base_type = child.get(0)
                            .and_then(|e| e.value().as_string())
                            .unwrap_or("")
                            .to_string();
                    }
                    "rust_type" => {
                        typedef.rust_type = child.get(0)
                            .and_then(|e| e.value().as_string())
                            .map(String::from);
                    }
                    "typescript_type" => {
                        typedef.typescript_type = child.get(0)
                            .and_then(|e| e.value().as_string())
                            .map(String::from);
                    }
                    "format" => {
                        typedef.format = child.get(0)
                            .and_then(|e| e.value().as_string())
                            .map(String::from);
                    }
                    "pattern" => {
                        typedef.pattern = child.get(0)
                            .and_then(|e| e.value().as_string())
                            .map(String::from);
                    }
                    _ => {}
                }
            }
        }

        Ok(typedef)
    }
}

impl Default for SchemaParser {
    fn default() -> Self {
        Self::new()
    }
}