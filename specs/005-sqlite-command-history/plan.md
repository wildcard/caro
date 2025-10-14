# Technical Implementation Plan: SQLite Command History Storage

**Feature**: SQLite Command History Storage  
**Feature Branch**: `005-sqlite-command-history`  
**Specification**: `spec.md`  
**Created**: 2025-10-14  
**Status**: Planning  

---

## Phase 0: Research & Resolution *(prerequisite)*

### Technical Unknowns & Decisions

#### Database Technology Selection
- **Choice**: SQLite 3 via `rusqlite` crate
- **Rationale**: 
  - Zero-config embedded database perfect for local CLI tools
  - Excellent Rust ecosystem support with `rusqlite`
  - Built-in full-text search (FTS5) for command search functionality
  - Cross-platform file format ensures portability
  - Minimal dependencies align with project constitution (simplicity principle)

#### Storage Location Strategy  
- **Choice**: User-specific history in platform-appropriate cache directory
- **Implementation**: Use `directories` crate for cross-platform cache directories
  - Linux: `~/.cache/cmdai/history.db`
  - macOS: `~/Library/Caches/cmdai/history.db`  
  - Windows: `%LOCALAPPDATA%/cmdai/history.db`

#### Database Schema Design
- **Primary Table**: `command_history` with comprehensive metadata
- **Search Support**: SQLite FTS5 virtual table for fast text search
- **Performance**: Indexed columns for timestamp and backend filtering
- **Privacy**: Separate `sensitive_data` field with encryption option

#### Performance Requirements Analysis
- **Target**: <10ms writes, <50ms searches, <100ms complex queries
- **Strategy**: 
  - Prepared statements for all database operations
  - Connection pooling using `r2d2_sqlite` for concurrent access
  - Lazy database initialization to maintain <100ms startup time
  - Periodic VACUUM operations during idle time

### Constitution Check

#### I. Simplicity ✅
- **Single data model**: `CommandHistoryEntry` with clear relationships
- **Direct rusqlite usage**: No ORM or additional abstraction layers
- **Clear API**: Simple `HistoryManager` trait with essential operations only

#### II. Library-First Architecture ✅  
- **New module**: `cmdai::history` with public API
- **Standalone testing**: All history operations testable independently
- **Clean separation**: History storage separate from command generation logic

#### III. Test-First (NON-NEGOTIABLE) ✅
- **Contract tests**: Database schema and migration operations
- **Integration tests**: Full workflow with real SQLite database  
- **Property tests**: Search functionality with random data sets
- **Performance tests**: Benchmark database operations against requirements

#### IV. Safety-First Development ✅
- **No unsafe code**: Pure Rust implementation with rusqlite safety guarantees
- **SQL injection prevention**: Prepared statements and parameter binding
- **Error handling**: Comprehensive database error recovery strategies
- **Privacy protection**: Configurable sensitive data filtering

#### V. Observability & Versioning ✅
- **Structured logging**: All database operations logged with tracing
- **Error context**: Detailed database error reporting with file paths
- **Performance monitoring**: Query timing and database size tracking
- **Schema versioning**: Migration system for future database changes

---

## Phase 1: Architecture & Contracts

### Core Architecture

#### Module Structure
```
src/history/
├── mod.rs              # Public API exports
├── manager.rs          # HistoryManager trait and implementation  
├── models.rs           # CommandHistoryEntry and related types
├── storage.rs          # SQLite database operations
├── search.rs           # Full-text search implementation
└── migrations.rs       # Database schema and migration logic
```

#### Data Model Contracts

##### CommandHistoryEntry
```rust
pub struct CommandHistoryEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub natural_language_input: String,
    pub generated_command: String,
    pub backend_used: String,
    pub success_status: CommandStatus,
    pub generation_time_ms: u32,
    pub working_directory: Option<String>,
    pub shell_environment: Option<String>,
    pub user_context: Option<serde_json::Value>,
}

pub enum CommandStatus {
    Generated,
    Executed,
    Failed,
    Cancelled,
}
```

##### HistoryManager Trait
```rust
#[async_trait]
pub trait HistoryManager {
    async fn store_command(&self, entry: CommandHistoryEntry) -> Result<i64>;
    async fn search_commands(&self, query: &str, filters: SearchFilters) -> Result<Vec<CommandHistoryEntry>>;
    async fn get_recent_commands(&self, limit: usize) -> Result<Vec<CommandHistoryEntry>>;
    async fn get_command_by_id(&self, id: i64) -> Result<Option<CommandHistoryEntry>>;
    async fn delete_command(&self, id: i64) -> Result<bool>;
    async fn export_history(&self, format: ExportFormat) -> Result<String>;
    async fn cleanup_old_entries(&self, policy: RetentionPolicy) -> Result<usize>;
}
```

### Database Schema

#### Core Tables
```sql
-- Main command history storage
CREATE TABLE command_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    natural_language_input TEXT NOT NULL,
    generated_command TEXT NOT NULL,
    backend_used TEXT NOT NULL,
    success_status TEXT NOT NULL,
    generation_time_ms INTEGER NOT NULL,
    working_directory TEXT,
    shell_environment TEXT,
    user_context TEXT, -- JSON blob
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Full-text search table for fast command search
CREATE VIRTUAL TABLE command_search USING fts5(
    natural_language_input,
    generated_command,
    content='command_history',
    content_rowid='id'
);

-- Indexes for performance
CREATE INDEX idx_timestamp ON command_history(timestamp);
CREATE INDEX idx_backend ON command_history(backend_used);
CREATE INDEX idx_status ON command_history(success_status);
```

### API Integration Points

#### CLI Integration
```rust
// Add to main.rs command handling
if args.history {
    let history_manager = SqliteHistoryManager::new(&config.cache_dir)?;
    display_command_history(history_manager, args.search_filter).await?;
}

if args.search.is_some() {
    let history_manager = SqliteHistoryManager::new(&config.cache_dir)?;
    search_and_display(history_manager, args.search.unwrap()).await?;
}
```

#### Backend Integration
```rust
// Modify command generation flow to store results
async fn generate_command_with_history(
    request: &CommandRequest,
    backend: &dyn ModelBackend,
    history_manager: &dyn HistoryManager,
) -> Result<GeneratedCommand> {
    let start_time = Instant::now();
    let result = backend.generate_command(request).await;
    let generation_time = start_time.elapsed().as_millis() as u32;
    
    if let Ok(ref command) = result {
        let entry = CommandHistoryEntry {
            timestamp: Utc::now(),
            natural_language_input: request.prompt.clone(),
            generated_command: command.command.clone(),
            backend_used: backend.name().to_string(),
            success_status: CommandStatus::Generated,
            generation_time_ms: generation_time,
            working_directory: request.working_directory.clone(),
            shell_environment: request.shell_type.map(|s| s.to_string()),
            user_context: None,
            ..Default::default()
        };
        
        if let Err(e) = history_manager.store_command(entry).await {
            tracing::warn!("Failed to store command history: {}", e);
        }
    }
    
    result
}
```

---

## Phase 2: Task Generation Strategy

### Implementation Phases
1. **Models & Types** (T001-T010): Define all data structures and enums
2. **Database Schema** (T011-T020): Create tables, indexes, and migration system  
3. **Core Storage** (T021-T030): Implement basic CRUD operations
4. **Search Functionality** (T031-T040): Add full-text search with FTS5
5. **CLI Integration** (T041-T050): Add history commands and integration
6. **Performance Optimization** (T051-T055): Benchmarking and optimization

### Key Dependencies
- **Sequential**: Models → Schema → Storage → Search → CLI Integration
- **Parallel Opportunities**: 
  - [P] Model definitions can be built in parallel with schema design
  - [P] Search functionality can be developed in parallel with basic storage
  - [P] CLI integration can be designed in parallel with core implementation

### Risk Mitigation
- **Database corruption**: Implement automatic backup and recovery
- **Performance degradation**: Continuous benchmarking with performance gates
- **Cross-platform compatibility**: Extensive testing on all target platforms
- **Privacy concerns**: Configurable data filtering and encryption options

---

## Progress Tracking

### Phase 0 Status ✅
- [x] Technical decisions documented
- [x] Constitution compliance verified  
- [x] Architecture contracts defined
- [x] Performance requirements specified

### Phase 1 Status 🟡
- [x] Core architecture designed
- [x] Data model contracts created
- [x] Database schema specified
- [x] API integration points planned
- [ ] Contract generation (pending task phase)

### Phase 2 Status ⏳
- [ ] Task breakdown (next step)
- [ ] Implementation planning
- [ ] TDD cycle preparation

---

## Dependencies & Integration

### External Dependencies
- `rusqlite` - SQLite database interface
- `r2d2_sqlite` - Connection pooling
- `serde_json` - JSON serialization for user context
- `chrono` - DateTime handling

### Internal Dependencies  
- `cmdai::models` - Core data types (CommandRequest, GeneratedCommand)
- `cmdai::config` - Configuration management
- `cmdai::cli` - Command line interface integration

### Integration Requirements
- **Backward compatibility**: History system must not affect existing command generation
- **Performance requirements**: Must maintain <100ms startup time, <2s first inference
- **Configuration**: History enabled by default, but can be disabled via config
- **Privacy**: Sensitive data filtering must be configurable and secure

---

*This plan provides the technical foundation for implementing SQLite command history storage while maintaining alignment with the project constitution and user requirements.*