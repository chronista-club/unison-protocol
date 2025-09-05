use anyhow::Result;
use crate::parser::{ParsedSchema, TypeRegistry};

pub mod rust;
pub mod typescript;

pub use rust::RustGenerator;
pub use typescript::TypeScriptGenerator;

/// Trait for code generators
pub trait CodeGenerator {
    /// Generate code from a parsed schema
    fn generate(&self, schema: &ParsedSchema, type_registry: &TypeRegistry) -> Result<String>;
    
    /// Generate code and write to file
    fn generate_to_file(&self, schema: &ParsedSchema, type_registry: &TypeRegistry, path: &str) -> Result<()> {
        let code = self.generate(schema, type_registry)?;
        std::fs::write(path, code)?;
        Ok(())
    }
}