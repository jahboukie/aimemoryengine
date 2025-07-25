# 🧠 Memory Engine MVP Sprint - Phase 1-2 Only

## Sprint Goal
Build a **local-only Memory Engine** that provides persistent project memory for AI coding assistants. Focus on core functionality that solves the "AI amnesia" problem for serious developers.

## What We're Building (Phase 1-2 Only)
- **Memory Engine Core** - Persistent project knowledge graph
- **File Analysis** - Real-time code parsing and relationship tracking
- **Local Storage** - SQLite-based persistence (no cloud)
- **Basic CLI** - Simple commands to interact with memory

## What We're NOT Building (Phase 3-4)
- ❌ Execution Engine (comes later)
- ❌ Cloud integration
- ❌ Advanced AI orchestration
- ❌ Enterprise features
- ❌ Plugin ecosystem

## Technical Architecture (Simplified)

```
┌─────────────────────────────────────────┐
│           Memory Engine MVP             │
├─────────────────────────────────────────┤
│  CLI (Node.js)     │  Core (Rust)       │
│  ┌─────────────┐   │  ┌─────────────┐   │
│  │ codecontext │   │  │ Memory      │   │
│  │ commands    │   │  │ Engine      │   │
│  └─────────────┘   │  └─────────────┘   │
├─────────────────────────────────────────┤
│  File Watcher      │  AST Parser        │
│  ┌─────────────┐   │  ┌─────────────┐   │
│  │ Real-time   │   │  │ JS/TS/Py    │   │
│  │ Changes     │   │  │ Analysis    │   │
│  └─────────────┘   │  └─────────────┘   │
├─────────────────────────────────────────┤
│            SQLite Database              │
│  ┌─────────────────────────────────────┐ │
│  │ Project Knowledge Graph             │ │
│  │ - Code Entities                     │ │
│  │ - Relationships                     │ │
│  │ - Change History                    │ │
│  └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

## Sprint Breakdown (4 weeks)

### Week 1: Foundation
**Goal**: Get basic project structure working

#### Day 1-2: Project Setup
- [ ] Initialize Rust workspace (`memory-engine/`)
- [ ] Set up Node.js CLI wrapper (`cli/`)
- [ ] Configure SQLite schema
- [ ] Basic build system (Cargo + npm)

#### Day 3-5: Core Data Structures
```rust
// Core structures we need to implement
pub struct CodeEntity {
    id: String,
    name: String,
    entity_type: EntityType, // Function, Class, Module, Variable
    file_path: String,
    line_start: u32,
    line_end: u32,
    metadata: HashMap<String, String>,
}

pub struct Relationship {
    from_entity: String,
    to_entity: String,
    relationship_type: RelationType, // Calls, Imports, Extends, Uses
    metadata: HashMap<String, String>,
}

pub struct ProjectMemory {
    entities: HashMap<String, CodeEntity>,
    relationships: Vec<Relationship>,
    file_hashes: HashMap<String, String>, // For change detection
}
```

### Week 2: File Analysis
**Goal**: Parse code and extract meaningful entities

#### Day 1-3: AST Parsing
- [ ] Integrate tree-sitter for JavaScript/TypeScript
- [ ] Extract functions, classes, imports
- [ ] Build relationship graph from code
- [ ] Handle basic Python files

#### Day 4-5: File System Watcher
- [ ] Real-time file change detection
- [ ] Incremental memory updates
- [ ] Handle file renames/deletions
- [ ] Batch processing for performance

### Week 3: Persistence & Querying
**Goal**: Store and retrieve project memory efficiently

#### Day 1-3: SQLite Integration
```sql
-- Core schema we need
CREATE TABLE entities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_start INTEGER,
    line_end INTEGER,
    metadata TEXT -- JSON
);

CREATE TABLE relationships (
    id INTEGER PRIMARY KEY,
    from_entity TEXT,
    to_entity TEXT,
    relationship_type TEXT,
    metadata TEXT -- JSON
);

CREATE INDEX idx_entities_file ON entities(file_path);
CREATE INDEX idx_relationships_from ON relationships(from_entity);
```

#### Day 4-5: Basic Queries
- [ ] Find all entities in a file
- [ ] Get relationships for an entity
- [ ] Search entities by name/type
- [ ] Basic performance optimization

### Week 4: CLI & Integration
**Goal**: Make it usable for developers

#### Day 1-3: Command Line Interface
```bash
# Commands we need to implement
codecontext init          # Initialize project memory
codecontext status        # Show memory statistics
codecontext analyze <file> # Analyze specific file
codecontext query <term>  # Search project context
codecontext reset         # Clear memory (for debugging)
```

#### Day 4-5: Basic AI Integration
- [ ] Simple JSON API for context queries
- [ ] Memory storage for AI learnings
- [ ] Context window optimization
- [ ] Basic error handling

## Success Criteria

### Technical Metrics
- [ ] **Parse Speed**: <1 second for typical JS/TS file
- [ ] **Memory Usage**: <50MB for medium project (1000 files)
- [ ] **Query Speed**: <100ms for context searches
- [ ] **Accuracy**: >95% entity extraction accuracy

### User Experience
- [ ] **Installation**: Single command install
- [ ] **Initialization**: `codecontext init` works in any project
- [ ] **Real-time**: Changes reflected within 1 second
- [ ] **Reliability**: No crashes on common codebases

### Dogfooding Validation
- [ ] Use tool to build itself
- [ ] Track our own development patterns
- [ ] Memory persists across sessions
- [ ] Provides useful context for AI assistants

## Technology Stack (Minimal)

### Core Engine (Rust)
- `tokio` - Async runtime
- `rusqlite` - SQLite integration
- `tree-sitter` - Code parsing
- `notify` - File system watching
- `serde` - Serialization

### CLI Wrapper (Node.js)
- `commander` - CLI framework
- `chalk` - Terminal colors
- `ora` - Loading spinners
- `node-ffi-napi` - Rust integration

### Database
- **SQLite** - Local, fast, reliable
- No external dependencies
- Works offline

## Deliverables

### End of Sprint
1. **Working CLI tool** that can be installed locally
2. **Memory persistence** across development sessions  
3. **Real-time tracking** of code changes
4. **Basic context queries** for AI assistants
5. **Dogfooding validation** - we used it to build itself

### Not Included (Future Phases)
- Code execution capabilities
- Cloud synchronization
- Advanced AI orchestration
- Multi-language support beyond JS/TS/Python
- Enterprise features

## Next Steps After Sprint
Once Phase 1-2 is complete, we'll evaluate:
1. **User feedback** from dogfooding
2. **Performance** on real codebases
3. **Value proposition** validation
4. **Decision**: Add Execution Engine (Phase 3) or ship MVP

This focused sprint gives us a solid foundation and real validation before expanding scope.
