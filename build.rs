use std::env;
use std::fs;
use std::path::{Path, PathBuf};

mod build_certs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate development certificates for embedding
    build_certs::generate_dev_certs()?;
    
    // Get the output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    
    // Load and parse schemas
    let schemas_dir = Path::new("schemas");
    let mut all_schemas = String::new();
    
    // First, load common.kdl
    if let Ok(common) = fs::read_to_string(schemas_dir.join("common.kdl")) {
        all_schemas.push_str(&common);
        all_schemas.push_str("\n\n");
    }
    
    // Then load the main protocol definition
    if let Ok(main_schema) = fs::read_to_string(schemas_dir.join("diarkis_devtools.kdl")) {
        all_schemas.push_str(&main_schema);
    }
    
    // Generate Rust code
    let generated_rust = generate_rust_code(&all_schemas)?;
    fs::write(out_dir.join("generated.rs"), generated_rust)?;
    
    // Generate TypeScript definitions
    let generated_ts = generate_typescript_code(&all_schemas)?;
    
    // Create TypeScript output directory if it doesn't exist
    let ts_out_dir = Path::new("../../webui-ts/src/generated");
    if !ts_out_dir.exists() {
        fs::create_dir_all(ts_out_dir)?;
    }
    fs::write(ts_out_dir.join("protocol.ts"), generated_ts)?;
    
    // Tell cargo to rerun this build script if schemas change
    println!("cargo:rerun-if-changed=schemas/common.kdl");
    println!("cargo:rerun-if-changed=schemas/diarkis_devtools.kdl");
    
    Ok(())
}

fn generate_rust_code(_schema_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // For now, generate a placeholder
    // In production, this would use the actual parser and generator
    Ok(r#"
// Auto-generated protocol definitions
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Placeholder for generated types
// This will be replaced with actual generated code from KDL schemas

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationSession {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub id: Uuid,
    pub source_text: String,
    pub target_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: DateTime<Utc>,
}
"#.to_string())
}

fn generate_typescript_code(_schema_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // For now, generate a placeholder
    // In production, this would use the actual parser and generator
    Ok(r#"// Auto-generated TypeScript protocol definitions
// DO NOT EDIT MANUALLY

export type UUID = string;
export type Timestamp = string;
export type LanguageCode = string;

export interface TranslationSession {
  id: UUID;
  name: string;
  status: 'draft' | 'pending' | 'in_progress' | 'completed' | 'failed' | 'archived';
  created_at: Timestamp;
}

export interface Translation {
  id: UUID;
  source_text: string;
  target_text: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  created_at: Timestamp;
}
"#.to_string())
}