use crate::{CodeEntity, Relationship, RelationshipQuery};
use std::collections::HashMap;

/// Core project memory that holds the knowledge graph
#[derive(Debug, Clone)]
pub struct ProjectMemory {
    pub entities: HashMap<String, CodeEntity>,
    pub relationships: Vec<Relationship>,
    pub file_hashes: HashMap<String, String>, // For change detection
    pub project_path: String,
}

impl ProjectMemory {
    pub fn new(project_path: String) -> Self {
        Self {
            entities: HashMap::new(),
            relationships: Vec::new(),
            file_hashes: HashMap::new(),
            project_path,
        }
    }

    /// Add or update a code entity
    pub fn add_entity(&mut self, entity: CodeEntity) {
        self.entities.insert(entity.id.clone(), entity);
    }

    /// Remove an entity and all its relationships
    pub fn remove_entity(&mut self, entity_id: &str) {
        self.entities.remove(entity_id);
        self.relationships.retain(|rel| {
            rel.from_entity != entity_id && rel.to_entity != entity_id
        });
    }

    /// Add a relationship between entities
    pub fn add_relationship(&mut self, relationship: Relationship) {
        // Check if relationship already exists
        let signature = relationship.get_signature();
        if !self.relationships.iter().any(|r| r.get_signature() == signature) {
            self.relationships.push(relationship);
        }
    }

    /// Find entities by name pattern
    pub fn find_entities_by_name(&self, pattern: &str) -> Vec<&CodeEntity> {
        self.entities
            .values()
            .filter(|entity| entity.name.contains(pattern))
            .collect()
    }

    /// Find entities in a specific file
    pub fn find_entities_in_file(&self, file_path: &str) -> Vec<&CodeEntity> {
        self.entities
            .values()
            .filter(|entity| entity.file_path == file_path)
            .collect()
    }

    /// Find relationships matching a query
    pub fn find_relationships(&self, query: &RelationshipQuery) -> Vec<&Relationship> {
        self.relationships
            .iter()
            .filter(|rel| query.matches(rel))
            .collect()
    }

    /// Get all entities that a given entity calls/uses
    pub fn get_dependencies(&self, entity_id: &str) -> Vec<&CodeEntity> {
        let mut dependencies = Vec::new();
        
        for rel in &self.relationships {
            if rel.from_entity == entity_id {
                if let Some(target_entity) = self.entities.get(&rel.to_entity) {
                    dependencies.push(target_entity);
                }
            }
        }
        
        dependencies
    }

    /// Get all entities that call/use a given entity
    pub fn get_dependents(&self, entity_id: &str) -> Vec<&CodeEntity> {
        let mut dependents = Vec::new();
        
        for rel in &self.relationships {
            if rel.to_entity == entity_id {
                if let Some(source_entity) = self.entities.get(&rel.from_entity) {
                    dependents.push(source_entity);
                }
            }
        }
        
        dependents
    }

    /// Update file hash for change detection
    pub fn update_file_hash(&mut self, file_path: String, hash: String) {
        self.file_hashes.insert(file_path, hash);
    }

    /// Check if file has changed
    pub fn has_file_changed(&self, file_path: &str, current_hash: &str) -> bool {
        match self.file_hashes.get(file_path) {
            Some(stored_hash) => stored_hash != current_hash,
            None => true, // File is new
        }
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            entity_count: self.entities.len(),
            relationship_count: self.relationships.len(),
            file_count: self.file_hashes.len(),
            project_path: self.project_path.clone(),
        }
    }

    /// Clear all memory (useful for testing)
    pub fn clear(&mut self) {
        self.entities.clear();
        self.relationships.clear();
        self.file_hashes.clear();
    }
}

/// Statistics about the project memory
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub entity_count: usize,
    pub relationship_count: usize,
    pub file_count: usize,
    pub project_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntityType;

    #[test]
    fn test_project_memory_creation() {
        let memory = ProjectMemory::new("/test/project".to_string());
        assert_eq!(memory.project_path, "/test/project");
        assert_eq!(memory.entities.len(), 0);
        assert_eq!(memory.relationships.len(), 0);
    }

    #[test]
    fn test_add_entity() {
        let mut memory = ProjectMemory::new("/test".to_string());
        let entity = CodeEntity::new(
            "test_func".to_string(),
            EntityType::Function,
            "test.js".to_string(),
            1, 10, 0, 20
        );
        let entity_id = entity.id.clone();
        
        memory.add_entity(entity);
        assert_eq!(memory.entities.len(), 1);
        assert!(memory.entities.contains_key(&entity_id));
    }

    #[test]
    fn test_find_entities_by_name() {
        let mut memory = ProjectMemory::new("/test".to_string());
        let entity = CodeEntity::new(
            "test_function".to_string(),
            EntityType::Function,
            "test.js".to_string(),
            1, 10, 0, 20
        );
        
        memory.add_entity(entity);
        let results = memory.find_entities_by_name("test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_function");
    }
}
