use crate::{ProjectMemory, CodeEntity, Relationship, EntityType, RelationType};
use anyhow::Result;
use rusqlite::{Connection, params, Transaction};
use std::collections::HashMap;

pub struct MemoryStorage {
    conn: Connection,
}

impl MemoryStorage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create entities table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line_start INTEGER NOT NULL,
                line_end INTEGER NOT NULL,
                column_start INTEGER NOT NULL,
                column_end INTEGER NOT NULL,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create relationships table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS relationships (
                id TEXT PRIMARY KEY,
                from_entity TEXT NOT NULL,
                to_entity TEXT NOT NULL,
                relationship_type TEXT NOT NULL,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create file_hashes table for change detection
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_hashes (
                file_path TEXT PRIMARY KEY,
                hash TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_entities_file ON entities(file_path)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_relationships_from ON relationships(from_entity)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_relationships_to ON relationships(to_entity)", [])?;

        Ok(Self { conn })
    }

    pub fn save_memory(&self, memory: &ProjectMemory) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        // Clear existing data for this project
        tx.execute("DELETE FROM entities", [])?;
        tx.execute("DELETE FROM relationships", [])?;
        tx.execute("DELETE FROM file_hashes", [])?;

        // Save entities
        for entity in memory.entities.values() {
            self.save_entity_in_tx(&tx, entity)?;
        }

        // Save relationships
        for relationship in &memory.relationships {
            self.save_relationship_in_tx(&tx, relationship)?;
        }

        // Save file hashes
        for (file_path, hash) in &memory.file_hashes {
            tx.execute(
                "INSERT INTO file_hashes (file_path, hash, updated_at) VALUES (?1, ?2, datetime('now'))",
                params![file_path, hash],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_memory(&self, project_path: &str) -> Result<ProjectMemory> {
        let mut memory = ProjectMemory::new(project_path.to_string());

        // Load entities
        let mut stmt = self.conn.prepare(
            "SELECT id, name, entity_type, file_path, line_start, line_end, column_start, column_end, metadata, created_at, updated_at FROM entities"
        )?;

        let entity_iter = stmt.query_map([], |row| {
            let entity_type_str: String = row.get(2)?;
            let entity_type = EntityType::from_str(&entity_type_str).unwrap_or(EntityType::Function);

            let metadata_json: String = row.get(8)?;
            let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json).unwrap_or_default();

            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            let mut entity = CodeEntity::new(
                row.get(1)?,
                entity_type,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            );

            entity.id = row.get(0)?;
            entity.metadata = metadata;
            entity.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            entity.updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);

            Ok(entity)
        })?;

        for entity in entity_iter {
            let entity = entity?;
            memory.entities.insert(entity.id.clone(), entity);
        }

        // Load relationships
        let mut stmt = self.conn.prepare(
            "SELECT id, from_entity, to_entity, relationship_type, metadata, created_at, updated_at FROM relationships"
        )?;

        let relationship_iter = stmt.query_map([], |row| {
            let rel_type_str: String = row.get(3)?;
            let rel_type = RelationType::from_str(&rel_type_str).unwrap_or(RelationType::Uses);

            let metadata_json: String = row.get(4)?;
            let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json).unwrap_or_default();

            let created_at_str: String = row.get(5)?;
            let updated_at_str: String = row.get(6)?;

            let mut relationship = Relationship::new(
                row.get(1)?,
                row.get(2)?,
                rel_type,
            );

            relationship.id = row.get(0)?;
            relationship.metadata = metadata;
            relationship.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            relationship.updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);

            Ok(relationship)
        })?;

        for relationship in relationship_iter {
            memory.relationships.push(relationship?);
        }

        // Load file hashes
        let mut stmt = self.conn.prepare("SELECT file_path, hash FROM file_hashes")?;
        let hash_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for hash_result in hash_iter {
            let (file_path, hash) = hash_result?;
            memory.file_hashes.insert(file_path, hash);
        }

        Ok(memory)
    }

    // Helper methods for transaction-based operations
    fn save_entity_in_tx(&self, tx: &Transaction, entity: &CodeEntity) -> Result<()> {
        let metadata_json = serde_json::to_string(&entity.metadata)?;
        let created_at = entity.created_at.to_rfc3339();
        let updated_at = entity.updated_at.to_rfc3339();

        tx.execute(
            "INSERT OR REPLACE INTO entities
             (id, name, entity_type, file_path, line_start, line_end, column_start, column_end, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entity.id,
                entity.name,
                entity.entity_type.as_str(),
                entity.file_path,
                entity.line_start,
                entity.line_end,
                entity.column_start,
                entity.column_end,
                metadata_json,
                created_at,
                updated_at
            ],
        )?;

        Ok(())
    }

    fn save_relationship_in_tx(&self, tx: &Transaction, relationship: &Relationship) -> Result<()> {
        let metadata_json = serde_json::to_string(&relationship.metadata)?;
        let created_at = relationship.created_at.to_rfc3339();
        let updated_at = relationship.updated_at.to_rfc3339();

        tx.execute(
            "INSERT OR REPLACE INTO relationships
             (id, from_entity, to_entity, relationship_type, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                relationship.id,
                relationship.from_entity,
                relationship.to_entity,
                relationship.relationship_type.as_str(),
                metadata_json,
                created_at,
                updated_at
            ],
        )?;

        Ok(())
    }

    // Query methods for specific use cases
    pub fn find_entities_by_file(&self, file_path: &str) -> Result<Vec<CodeEntity>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, entity_type, file_path, line_start, line_end, column_start, column_end, metadata, created_at, updated_at
             FROM entities WHERE file_path = ?1 ORDER BY line_start"
        )?;

        let entity_iter = stmt.query_map([file_path], |row| {
            let entity_type_str: String = row.get(2)?;
            let entity_type = EntityType::from_str(&entity_type_str).unwrap_or(EntityType::Function);

            let metadata_json: String = row.get(8)?;
            let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json).unwrap_or_default();

            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            let mut entity = CodeEntity::new(
                row.get(1)?,
                entity_type,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            );

            entity.id = row.get(0)?;
            entity.metadata = metadata;
            entity.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            entity.updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);

            Ok(entity)
        })?;

        let mut entities = Vec::new();
        for entity in entity_iter {
            entities.push(entity?);
        }

        Ok(entities)
    }

    pub fn find_entities_by_name(&self, pattern: &str) -> Result<Vec<CodeEntity>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, entity_type, file_path, line_start, line_end, column_start, column_end, metadata, created_at, updated_at
             FROM entities WHERE name LIKE ?1 ORDER BY name"
        )?;

        let search_pattern = format!("%{}%", pattern);
        let entity_iter = stmt.query_map([search_pattern], |row| {
            let entity_type_str: String = row.get(2)?;
            let entity_type = EntityType::from_str(&entity_type_str).unwrap_or(EntityType::Function);

            let metadata_json: String = row.get(8)?;
            let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json).unwrap_or_default();

            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;

            let mut entity = CodeEntity::new(
                row.get(1)?,
                entity_type,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            );

            entity.id = row.get(0)?;
            entity.metadata = metadata;
            entity.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            entity.updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);

            Ok(entity)
        })?;

        let mut entities = Vec::new();
        for entity in entity_iter {
            entities.push(entity?);
        }

        Ok(entities)
    }

    pub fn get_stats(&self) -> Result<(usize, usize, usize)> {
        let entity_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM entities",
            [],
            |row| row.get(0)
        )?;

        let relationship_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM relationships",
            [],
            |row| row.get(0)
        )?;

        let file_count: usize = self.conn.query_row(
            "SELECT COUNT(DISTINCT file_path) FROM entities",
            [],
            |row| row.get(0)
        )?;

        Ok((entity_count, relationship_count, file_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_creation() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let storage = MemoryStorage::new(temp_file.path().to_str().unwrap())?;

        // Test that tables were created by checking stats
        let (entity_count, relationship_count, file_count) = storage.get_stats()?;
        assert_eq!(entity_count, 0);
        assert_eq!(relationship_count, 0);
        assert_eq!(file_count, 0);

        Ok(())
    }

    #[test]
    fn test_memory_persistence() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file.path().to_str().unwrap();

        // Create and populate memory
        let mut memory = ProjectMemory::new("/test/project".to_string());

        let entity1 = CodeEntity::new(
            "testFunction".to_string(),
            EntityType::Function,
            "test.js".to_string(),
            10, 20, 0, 15
        );

        let entity2 = CodeEntity::new(
            "TestClass".to_string(),
            EntityType::Class,
            "test.js".to_string(),
            25, 35, 0, 20
        );

        memory.add_entity(entity1.clone());
        memory.add_entity(entity2.clone());
        memory.update_file_hash("test.js".to_string(), "abc123".to_string());

        // Save to storage
        {
            let storage = MemoryStorage::new(db_path)?;
            storage.save_memory(&memory)?;
        }

        // Load from storage in new instance
        {
            let storage = MemoryStorage::new(db_path)?;
            let loaded_memory = storage.load_memory("/test/project")?;

            assert_eq!(loaded_memory.entities.len(), 2);
            assert_eq!(loaded_memory.file_hashes.len(), 1);
            assert_eq!(loaded_memory.file_hashes.get("test.js"), Some(&"abc123".to_string()));

            // Verify entities were loaded correctly
            let entities: Vec<_> = loaded_memory.entities.values().collect();
            assert!(entities.iter().any(|e| e.name == "testFunction"));
            assert!(entities.iter().any(|e| e.name == "TestClass"));
        }

        Ok(())
    }

    #[test]
    fn test_find_entities_by_file() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let storage = MemoryStorage::new(temp_file.path().to_str().unwrap())?;

        let mut memory = ProjectMemory::new("/test".to_string());

        let entity1 = CodeEntity::new(
            "func1".to_string(),
            EntityType::Function,
            "file1.js".to_string(),
            10, 20, 0, 15
        );

        let entity2 = CodeEntity::new(
            "func2".to_string(),
            EntityType::Function,
            "file2.js".to_string(),
            10, 20, 0, 15
        );

        memory.add_entity(entity1);
        memory.add_entity(entity2);
        storage.save_memory(&memory)?;

        // Test file-specific queries
        let file1_entities = storage.find_entities_by_file("file1.js")?;
        assert_eq!(file1_entities.len(), 1);
        assert_eq!(file1_entities[0].name, "func1");

        let file2_entities = storage.find_entities_by_file("file2.js")?;
        assert_eq!(file2_entities.len(), 1);
        assert_eq!(file2_entities[0].name, "func2");

        Ok(())
    }
}
