use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Types of code entities we track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Function,
    Class,
    Module,
    Variable,
    Import,
    Export,
    Interface,
    Type,
    Constant,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Function => "function",
            EntityType::Class => "class",
            EntityType::Module => "module",
            EntityType::Variable => "variable",
            EntityType::Import => "import",
            EntityType::Export => "export",
            EntityType::Interface => "interface",
            EntityType::Type => "type",
            EntityType::Constant => "constant",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "function" => Some(EntityType::Function),
            "class" => Some(EntityType::Class),
            "module" => Some(EntityType::Module),
            "variable" => Some(EntityType::Variable),
            "import" => Some(EntityType::Import),
            "export" => Some(EntityType::Export),
            "interface" => Some(EntityType::Interface),
            "type" => Some(EntityType::Type),
            "constant" => Some(EntityType::Constant),
            _ => None,
        }
    }
}

/// Core code entity that represents any trackable element in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub column_start: u32,
    pub column_end: u32,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CodeEntity {
    pub fn new(
        name: String,
        entity_type: EntityType,
        file_path: String,
        line_start: u32,
        line_end: u32,
        column_start: u32,
        column_end: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            entity_type,
            file_path,
            line_start,
            line_end,
            column_start,
            column_end,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn update_position(&mut self, line_start: u32, line_end: u32, column_start: u32, column_end: u32) {
        self.line_start = line_start;
        self.line_end = line_end;
        self.column_start = column_start;
        self.column_end = column_end;
        self.updated_at = Utc::now();
    }

    pub fn get_signature(&self) -> String {
        format!("{}:{}:{}:{}", self.file_path, self.entity_type.as_str(), self.name, self.line_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = CodeEntity::new(
            "test_function".to_string(),
            EntityType::Function,
            "src/main.rs".to_string(),
            10,
            20,
            0,
            10,
        );

        assert_eq!(entity.name, "test_function");
        assert_eq!(entity.entity_type, EntityType::Function);
        assert_eq!(entity.file_path, "src/main.rs");
        assert_eq!(entity.line_start, 10);
        assert_eq!(entity.line_end, 20);
    }

    #[test]
    fn test_entity_type_conversion() {
        assert_eq!(EntityType::Function.as_str(), "function");
        assert_eq!(EntityType::from_str("function"), Some(EntityType::Function));
        assert_eq!(EntityType::from_str("invalid"), None);
    }
}
