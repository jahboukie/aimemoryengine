use crate::{CodeEntity, Relationship, EntityType};
use anyhow::Result;
use std::fs;
use regex::Regex;

pub struct CodeParser {
    // JavaScript/TypeScript patterns
    js_function_regex: Regex,
    js_class_regex: Regex,
    js_import_regex: Regex,
    js_variable_regex: Regex,

    // Rust patterns
    rust_function_regex: Regex,
    rust_struct_regex: Regex,
    rust_impl_regex: Regex,
    rust_trait_regex: Regex,
    rust_enum_regex: Regex,
    rust_use_regex: Regex,
    rust_mod_regex: Regex,
    rust_const_regex: Regex,
}

impl CodeParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // JavaScript/TypeScript patterns
            js_function_regex: Regex::new(r"(?m)^(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(")?,
            js_class_regex: Regex::new(r"(?m)^(?:export\s+)?class\s+(\w+)")?,
            js_import_regex: Regex::new(r#"(?m)^import\s+.*?from\s+['"]([^'"]+)['"]"#)?,
            js_variable_regex: Regex::new(r"(?m)^(?:const|let|var)\s+(\w+)")?,

            // Rust patterns
            rust_function_regex: Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")?,
            rust_struct_regex: Regex::new(r"(?m)^\s*(?:pub\s+)?struct\s+(\w+)")?,
            rust_impl_regex: Regex::new(r"(?m)^\s*impl(?:<[^>]*>)?\s+(?:\w+\s+for\s+)?(\w+)")?,
            rust_trait_regex: Regex::new(r"(?m)^\s*(?:pub\s+)?trait\s+(\w+)")?,
            rust_enum_regex: Regex::new(r"(?m)^\s*(?:pub\s+)?enum\s+(\w+)")?,
            rust_use_regex: Regex::new(r"(?m)^\s*use\s+([^;]+);")?,
            rust_mod_regex: Regex::new(r"(?m)^\s*(?:pub\s+)?mod\s+(\w+)")?,
            rust_const_regex: Regex::new(r"(?m)^\s*(?:pub\s+)?const\s+(\w+)")?,
        })
    }

    pub fn parse_file(&self, file_path: &str) -> Result<(Vec<CodeEntity>, Vec<Relationship>)> {
        let content = fs::read_to_string(file_path)?;
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "js" | "jsx" | "ts" | "tsx" => self.parse_javascript_like(&content, file_path),
            "py" => self.parse_python(&content, file_path),
            "rs" => self.parse_rust(&content, file_path),
            _ => Ok((Vec::new(), Vec::new())),
        }
    }

    fn parse_javascript_like(&self, content: &str, file_path: &str) -> Result<(Vec<CodeEntity>, Vec<Relationship>)> {
        let mut entities = Vec::new();
        let relationships = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = self.js_function_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Function,
                        file_path.to_string(),
                        line_num as u32 + 1,
                        line_num as u32 + 1,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            if let Some(captures) = self.js_class_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Class,
                        file_path.to_string(),
                        line_num as u32 + 1,
                        line_num as u32 + 1,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            if let Some(captures) = self.js_import_regex.captures(line) {
                if let Some(import_path) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        import_path.as_str().to_string(),
                        EntityType::Import,
                        file_path.to_string(),
                        line_num as u32 + 1,
                        line_num as u32 + 1,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            if let Some(captures) = self.js_variable_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Variable,
                        file_path.to_string(),
                        line_num as u32 + 1,
                        line_num as u32 + 1,
                        0,
                        line.len() as u32,
                    ));
                }
            }
        }

        Ok((entities, relationships))
    }

    fn parse_python(&self, content: &str, file_path: &str) -> Result<(Vec<CodeEntity>, Vec<Relationship>)> {
        let mut entities = Vec::new();
        let relationships = Vec::new();

        let py_function_regex = Regex::new(r"(?m)^def\s+(\w+)\s*\(")?;
        let py_class_regex = Regex::new(r"(?m)^class\s+(\w+)")?;
        let py_import_regex = Regex::new(r"(?m)^(?:from\s+(\S+)\s+)?import\s+(\S+)")?;

        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = py_function_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Function,
                        file_path.to_string(),
                        line_num as u32 + 1,
                        line_num as u32 + 1,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            if let Some(captures) = py_class_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Class,
                        file_path.to_string(),
                        line_num as u32 + 1,
                        line_num as u32 + 1,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            if let Some(captures) = py_import_regex.captures(line) {
                let import_name = if let Some(from_module) = captures.get(1) {
                    format!("{}.{}", from_module.as_str(), captures.get(2).unwrap().as_str())
                } else {
                    captures.get(2).unwrap().as_str().to_string()
                };

                entities.push(CodeEntity::new(
                    import_name,
                    EntityType::Import,
                    file_path.to_string(),
                    line_num as u32 + 1,
                    line_num as u32 + 1,
                    0,
                    line.len() as u32,
                ));
            }
        }

        Ok((entities, relationships))
    }
    fn parse_rust(&self, content: &str, file_path: &str) -> Result<(Vec<CodeEntity>, Vec<Relationship>)> {
        let mut entities = Vec::new();
        let relationships = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_number = (line_num + 1) as u32;

            // Parse Rust functions
            if let Some(captures) = self.rust_function_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Function,
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            // Parse Rust structs
            if let Some(captures) = self.rust_struct_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Class, // Using Class for structs
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            // Parse Rust traits
            if let Some(captures) = self.rust_trait_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Class, // Using Class for traits
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            // Parse Rust enums
            if let Some(captures) = self.rust_enum_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Class, // Using Class for enums
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            // Parse Rust use statements (imports)
            if let Some(captures) = self.rust_use_regex.captures(line) {
                if let Some(import_path) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        import_path.as_str().to_string(),
                        EntityType::Import,
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            // Parse Rust modules
            if let Some(captures) = self.rust_mod_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Module,
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            // Parse Rust constants
            if let Some(captures) = self.rust_const_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        name.as_str().to_string(),
                        EntityType::Variable,
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }

            // Parse Rust impl blocks
            if let Some(captures) = self.rust_impl_regex.captures(line) {
                if let Some(name) = captures.get(1) {
                    entities.push(CodeEntity::new(
                        format!("impl {}", name.as_str()),
                        EntityType::Class, // Using Class for impl blocks
                        file_path.to_string(),
                        line_number,
                        line_number,
                        0,
                        line.len() as u32,
                    ));
                }
            }
        }

        Ok((entities, relationships))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_javascript() -> Result<()> {
        let parser = CodeParser::new()?;

        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "function testFunction() {{")?;
        writeln!(temp_file, "  return 'hello';")?;
        writeln!(temp_file, "}}")?;
        writeln!(temp_file, "class TestClass {{}}")?;
        writeln!(temp_file, "import React from 'react';")?;
        writeln!(temp_file, "const myVar = 42;")?;

        let temp_path = temp_file.path().with_extension("js");
        fs::copy(temp_file.path(), &temp_path)?;

        let (entities, _relationships) = parser.parse_file(temp_path.to_str().unwrap())?;

        assert!(!entities.is_empty());

        let functions: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Function).collect();
        let classes: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Class).collect();
        let imports: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Import).collect();
        let variables: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Variable).collect();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "testFunction");

        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "TestClass");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].name, "react");

        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0].name, "myVar");

        fs::remove_file(temp_path)?;

        Ok(())
    }

    #[test]
    fn test_rust_parsing() -> Result<()> {
        let parser = CodeParser::new()?;

        let rust_content = r#"
use std::collections::HashMap;
use anyhow::Result;

pub struct MemoryEngine {
    entities: HashMap<String, Entity>,
}

pub trait Analyzer {
    fn analyze(&self) -> Result<()>;
}

pub enum EntityType {
    Function,
    Struct,
    Trait,
}

impl MemoryEngine {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id.clone(), entity);
    }
}

pub fn create_parser() -> Result<CodeParser> {
    CodeParser::new()
}

pub const MAX_ENTITIES: usize = 1000;

pub mod storage {
    pub fn save_data() {}
}
"#;

        let temp_path = "test_rust_parsing.rs";
        fs::write(temp_path, rust_content)?;

        let (entities, _relationships) = parser.parse_file(temp_path)?;

        // Verify we found the expected entities
        let use_statements: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Import).collect();
        let structs: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Class && e.name == "MemoryEngine").collect();
        let traits: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Class && e.name == "Analyzer").collect();
        let enums: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Class && e.name == "EntityType").collect();
        let functions: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Function).collect();
        let constants: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Variable && e.name == "MAX_ENTITIES").collect();
        let modules: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Module).collect();

        assert!(use_statements.len() >= 2, "Should find use statements");
        assert_eq!(structs.len(), 1, "Should find MemoryEngine struct");
        assert_eq!(traits.len(), 1, "Should find Analyzer trait");
        assert_eq!(enums.len(), 1, "Should find EntityType enum");
        assert!(functions.len() >= 3, "Should find functions (new, add_entity, create_parser)");
        assert_eq!(constants.len(), 1, "Should find MAX_ENTITIES constant");
        assert_eq!(modules.len(), 1, "Should find storage module");

        fs::remove_file(temp_path)?;

        Ok(())
    }
}
