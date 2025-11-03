use crate::parser::{ParsedSchema, TypeRegistry};
use anyhow::Result;

pub mod rust;
pub mod typescript;

pub use rust::RustGenerator;
pub use typescript::TypeScriptGenerator;

/// コードジェネレータのトレイト
pub trait CodeGenerator {
    /// パース済みスキーマからコードを生成
    fn generate(&self, schema: &ParsedSchema, type_registry: &TypeRegistry) -> Result<String>;

    /// コードを生成してファイルに書き込み
    fn generate_to_file(
        &self,
        schema: &ParsedSchema,
        type_registry: &TypeRegistry,
        path: &str,
    ) -> Result<()> {
        let code = self.generate(schema, type_registry)?;
        std::fs::write(path, code)?;
        Ok(())
    }
}
