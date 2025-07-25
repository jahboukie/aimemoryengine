// SQLite storage functionality - to be implemented in Week 3
use crate::ProjectMemory;
use anyhow::Result;

pub struct MemoryStorage {
    // TODO: Add SQLite connection
}

impl MemoryStorage {
    pub fn new(_db_path: &str) -> Result<Self> {
        // TODO: Initialize SQLite database
        Ok(Self {})
    }

    pub fn save_memory(&self, _memory: &ProjectMemory) -> Result<()> {
        // TODO: Implement SQLite persistence
        Ok(())
    }

    pub fn load_memory(&self, _project_path: &str) -> Result<ProjectMemory> {
        // TODO: Implement SQLite loading
        Ok(ProjectMemory::new(_project_path.to_string()))
    }
}
