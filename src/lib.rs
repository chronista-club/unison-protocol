//! # Unison Protocol
//! 
//! **Unison Protocol** is a KDL-based type-safe communication framework that enables
//! seamless client-server communication with automatic code generation for multiple languages.
//! 
//! ## Features
//! 
//! - **Type-safe communication**: Automatic code generation from KDL protocol definitions
//! - **Multi-language support**: Generate client/server code for Rust, TypeScript, and more
//! - **WebSocket-based**: Real-time bidirectional communication
//! - **Schema validation**: Compile-time and runtime protocol validation
//! - **Async-first**: Built with async/await support from the ground up
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! # use anyhow::Result;
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! use unison_protocol::{UnisonProtocol, UnisonServer, UnisonServerExt};
//! use unison_protocol::network::NetworkError;
//!
//! // Load protocol schema
//! let mut protocol = UnisonProtocol::new();
//! // protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
//!
//! // Create server
//! let mut server = protocol.create_server();
//! server.register_handler("ping", |payload| {
//!     // Handle ping request
//!     Ok(serde_json::json!({"message": "pong"})) as Result<serde_json::Value, NetworkError>
//! });
//! // server.listen("127.0.0.1:8080").await?;
//! # Ok(())
//! # }
//! ```
//! 
//! ## Core Concepts
//! 
//! - **Protocol**: Top-level container defining services, messages, and types
//! - **Service**: Collection of RPC methods with request/response definitions  
//! - **Message**: Structured data types with typed fields
//! - **Method**: Individual RPC endpoints within a service
//! 
//! ## Generated Code
//! 
//! The protocol definitions are automatically compiled into strongly-typed
//! client and server code during the build process.

pub mod parser;
pub mod codegen;
pub mod network;

// Core module for protocol definitions
pub mod core;

// CGP-based context module
pub mod context;

// Re-export generated code
pub mod generated {
    // This will be populated by build.rs
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

// Re-export commonly used types
pub use parser::{SchemaParser, ParsedSchema};
pub use codegen::{RustGenerator, TypeScriptGenerator, CodeGenerator};
pub use network::{ProtocolClient, ProtocolServer, UnisonClient, UnisonServer, UnisonServerExt};

// Error types
pub use parser::ParseError as UnisonParseError;
pub use network::NetworkError as UnisonNetworkError;

/// Main entry point for Unison Protocol
pub struct UnisonProtocol {
    schemas: Vec<ParsedSchema>,
    parser: SchemaParser,
}

impl UnisonProtocol {
    /// Create a new Unison Protocol instance
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
            parser: SchemaParser::new(),
        }
    }
    
    /// Load a protocol schema from KDL string
    pub fn load_schema(&mut self, schema: &str) -> Result<(), UnisonParseError> {
        let parsed = self.parser.parse(schema)?;
        self.schemas.push(parsed);
        Ok(())
    }
    
    /// Generate Rust code from loaded schemas
    pub fn generate_rust_code(&self) -> Result<String, Box<dyn std::error::Error>> {
        let generator = RustGenerator::new();
        let type_registry = crate::parser::TypeRegistry::new(); // Temporary empty registry
        let mut code = String::new();
        
        for schema in &self.schemas {
            code.push_str(&generator.generate(schema, &type_registry)?);
            code.push('\n');
        }
        
        Ok(code)
    }
    
    /// Generate TypeScript code from loaded schemas  
    pub fn generate_typescript_code(&self) -> Result<String, Box<dyn std::error::Error>> {
        let generator = TypeScriptGenerator::new();
        let type_registry = crate::parser::TypeRegistry::new(); // Temporary empty registry
        let mut code = String::new();
        
        for schema in &self.schemas {
            code.push_str(&generator.generate(schema, &type_registry)?);
            code.push('\n');
        }
        
        Ok(code)
    }
    
    /// Create a new Unison client
    pub fn create_client(&self) -> Result<ProtocolClient, anyhow::Error> {
        Ok(ProtocolClient::new_default()?)
    }
    
    /// Create a new Unison server
    pub fn create_server(&self) -> ProtocolServer {
        ProtocolServer::new()
    }
}

impl Default for UnisonProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unison_protocol_creation() {
        let protocol = UnisonProtocol::new();
        assert_eq!(protocol.schemas.len(), 0);
    }
    
    #[test]
    fn test_parse_schema() {
        let schema = r#"
protocol "test" version="1.0.0" {
    namespace "test.protocol"
    description "Test protocol for unit testing"

    message "TestMessage" {
        description "Test message structure"
        field "id" type="string" required=#true description="Unique identifier"
        field "value" type="number" required=#false description="Optional numeric value"
    }

    service "TestService" {
        description "Test service for unit testing"

        method "test_method" {
            description "Test method"
            request "TestMessage"
            response "TestMessage"
        }
    }
}
        "#;
        
        let mut protocol = UnisonProtocol::new();
        let result = protocol.load_schema(schema);
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
        assert_eq!(protocol.schemas.len(), 1);
    }
    
    #[test]
    fn test_client_server_creation() {
        let protocol = UnisonProtocol::new();
        let _client = protocol.create_client().unwrap();
        let _server = protocol.create_server();
        // Test passes if no panics occur
    }
}