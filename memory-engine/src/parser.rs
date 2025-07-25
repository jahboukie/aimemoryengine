use crate::{CodeEntity, Relationship, EntityType};
use anyhow::Result;
use std::fs;
use regex::Regex;

pub struct CodeParser {
    function_regex: Regex,
    class_regex: Regex,
    import_regex: Regex,
    variable_regex: Regex,
}

impl CodeParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            function_regex: Regex::new(r"(?m)^(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(")?,
            class_regex: Regex::new(r"(?m)^(?:export\s+)?class\s+(\w+)")?,
            import_regex: Regex::new(r#"(?m)^import\s+.*?from\s+['"]([^'"]+)['"]"#)?,
            variable_regex: Regex::new(r"(?m)^(?:const|let|var)\s+(\w+)")?,
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
            _ => Ok((Vec::new(), Vec::new())),
        }
    }

    fn parse_javascript_like(&self, content: &str, file_path: &str) -> Result<(Vec<CodeEntity>, Vec<Relationship>)> {
        let mut entities = Vec::new();
        let relationships = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = self.function_regex.captures(line) {
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

            if let Some(captures) = self.class_regex.captures(line) {
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

            if let Some(captures) = self.import_regex.captures(line) {
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

            if let Some(captures) = self.variable_regex.captures(line) {
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
}
