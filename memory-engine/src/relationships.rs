use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Types of relationships between code entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    Calls,      // Function A calls Function B
    Imports,    // Module A imports Module B
    Extends,    // Class A extends Class B
    Implements, // Class A implements Interface B
    Uses,       // Entity A uses Entity B
    Defines,    // Module A defines Entity B
    References, // Entity A references Entity B
    Contains,   // Entity A contains Entity B
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::Calls => "calls",
            RelationType::Imports => "imports",
            RelationType::Extends => "extends",
            RelationType::Implements => "implements",
            RelationType::Uses => "uses",
            RelationType::Defines => "defines",
            RelationType::References => "references",
            RelationType::Contains => "contains",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "calls" => Some(RelationType::Calls),
            "imports" => Some(RelationType::Imports),
            "extends" => Some(RelationType::Extends),
            "implements" => Some(RelationType::Implements),
            "uses" => Some(RelationType::Uses),
            "defines" => Some(RelationType::Defines),
            "references" => Some(RelationType::References),
            "contains" => Some(RelationType::Contains),
            _ => None,
        }
    }
}

/// Represents a relationship between two code entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: RelationType,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Relationship {
    pub fn new(
        from_entity: String,
        to_entity: String,
        relationship_type: RelationType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            from_entity,
            to_entity,
            relationship_type,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
        self
    }

    pub fn get_signature(&self) -> String {
        format!(
            "{}->{}:{}",
            self.from_entity,
            self.to_entity,
            self.relationship_type.as_str()
        )
    }
}

/// Helper struct for querying relationships
#[derive(Debug, Clone)]
pub struct RelationshipQuery {
    pub from_entity: Option<String>,
    pub to_entity: Option<String>,
    pub relationship_type: Option<RelationType>,
}

impl RelationshipQuery {
    pub fn new() -> Self {
        Self {
            from_entity: None,
            to_entity: None,
            relationship_type: None,
        }
    }

    pub fn from_entity(mut self, entity_id: String) -> Self {
        self.from_entity = Some(entity_id);
        self
    }

    pub fn to_entity(mut self, entity_id: String) -> Self {
        self.to_entity = Some(entity_id);
        self
    }

    pub fn relationship_type(mut self, rel_type: RelationType) -> Self {
        self.relationship_type = Some(rel_type);
        self
    }

    pub fn matches(&self, relationship: &Relationship) -> bool {
        if let Some(ref from) = self.from_entity {
            if relationship.from_entity != *from {
                return false;
            }
        }

        if let Some(ref to) = self.to_entity {
            if relationship.to_entity != *to {
                return false;
            }
        }

        if let Some(ref rel_type) = self.relationship_type {
            if relationship.relationship_type != *rel_type {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let rel = Relationship::new(
            "entity1".to_string(),
            "entity2".to_string(),
            RelationType::Calls,
        );

        assert_eq!(rel.from_entity, "entity1");
        assert_eq!(rel.to_entity, "entity2");
        assert_eq!(rel.relationship_type, RelationType::Calls);
    }

    #[test]
    fn test_relationship_query() {
        let rel = Relationship::new(
            "entity1".to_string(),
            "entity2".to_string(),
            RelationType::Calls,
        );

        let query = RelationshipQuery::new()
            .from_entity("entity1".to_string())
            .relationship_type(RelationType::Calls);

        assert!(query.matches(&rel));

        let query2 = RelationshipQuery::new()
            .from_entity("entity3".to_string());

        assert!(!query2.matches(&rel));
    }
}
