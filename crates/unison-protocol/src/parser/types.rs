use std::collections::HashMap;

/// Type registry for managing custom types
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    typedefs: HashMap<String, TypeDefMapping>,
}

#[derive(Debug, Clone)]
struct TypeDefMapping {
    rust_type: String,
    typescript_type: String,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            typedefs: HashMap::new(),
        };

        // Register built-in types
        registry.register_builtin_types();
        registry
    }

    fn register_builtin_types(&mut self) {
        // Timestamp type
        self.register("timestamp", "chrono::DateTime<chrono::Utc>", "string");

        // UUID type
        self.register("uuid", "uuid::Uuid", "string");

        // Language code
        self.register("language_code", "String", "string");
    }

    pub fn register(&mut self, name: &str, rust_type: &str, typescript_type: &str) {
        self.typedefs.insert(
            name.to_string(),
            TypeDefMapping {
                rust_type: rust_type.to_string(),
                typescript_type: typescript_type.to_string(),
            },
        );
    }

    pub fn get_rust_type(&self, name: &str) -> Option<String> {
        self.typedefs.get(name).map(|t| t.rust_type.clone())
    }

    pub fn get_typescript_type(&self, name: &str) -> Option<String> {
        self.typedefs.get(name).map(|t| t.typescript_type.clone())
    }

    pub fn update_from_typedefs(&mut self, typedefs: &[super::schema::TypeDef]) {
        for typedef in typedefs {
            if let (Some(rust_type), Some(ts_type)) = (&typedef.rust_type, &typedef.typescript_type)
            {
                self.register(&typedef.name, rust_type, ts_type);
            }
        }
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
