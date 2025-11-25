# Implementation Tasks: SQLite Command History Storage

**Feature**: SQLite Command History Storage  
**Branch**: `005-sqlite-command-history`  
**Specification**: `spec.md`  
**Plan**: `plan.md`  
**Created**: 2025-10-14  

---

## Task Dependencies & Execution Order

### Phase 1: Setup & Models (T001-T010)
**Goal**: Establish foundation data structures and database setup

**T001**: Create history module structure with proper exports [P]
- **File**: `src/history/mod.rs`
- **Acceptance**: Module exports all public types and HistoryManager trait
- **Dependencies**: None
- **Test**: Compilation succeeds, public API accessible from main.rs

**T002**: Define CommandHistoryEntry model with all required fields [P]  
- **File**: `src/history/models.rs`
- **Acceptance**: Struct includes all FR-001 required fields with proper types
- **Dependencies**: None
- **Test**: Model serializes/deserializes correctly, all fields accessible

**T003**: Define CommandStatus enum with all states [P]
- **File**: `src/history/models.rs` 
- **Acceptance**: Enum covers Generated, Executed, Failed, Cancelled states
- **Dependencies**: None
- **Test**: Enum serialization and Display trait implementation work correctly

**T004**: Define SearchFilters struct for history queries [P]
- **File**: `src/history/models.rs`
- **Acceptance**: Supports date range, backend type, status, and text filters
- **Dependencies**: None  
- **Test**: Filter combinations work correctly in test scenarios

**T005**: Define HistoryManager trait with async interface [P]
- **File**: `src/history/manager.rs`
- **Acceptance**: Trait includes all FR operations (store, search, export, cleanup)
- **Dependencies**: T002 (CommandHistoryEntry)
- **Test**: Trait compiles with async signatures, mock implementation works

**T006**: Add rusqlite and r2d2_sqlite dependencies to Cargo.toml
- **File**: `Cargo.toml`
- **Acceptance**: Dependencies added with appropriate version constraints
- **Dependencies**: None
- **Test**: `cargo check` succeeds, dependencies resolve correctly

**T007**: Create database migration system structure [P]
- **File**: `src/history/migrations.rs`
- **Acceptance**: Migration trait and version tracking defined
- **Dependencies**: None
- **Test**: Migration framework compiles and mock migrations work

**T008**: Write contract test for CommandHistoryEntry serialization [P]
- **File**: `tests/contract/test_history_models.rs`
- **Acceptance**: Test validates JSON serialization/deserialization
- **Dependencies**: T002 (CommandHistoryEntry)
- **Test**: Test fails initially, passes after implementation

**T009**: Write contract test for HistoryManager trait interface [P]
- **File**: `tests/contract/test_history_manager.rs`
- **Acceptance**: Test validates trait method signatures and return types
- **Dependencies**: T005 (HistoryManager trait)
- **Test**: Test fails initially, passes after trait implementation

**T010**: Write failing integration test for end-to-end history workflow
- **File**: `tests/integration/test_history_workflow.rs`
- **Acceptance**: Test covers store → search → retrieve → export cycle
- **Dependencies**: T005 (HistoryManager trait)
- **Test**: Test fails initially, will pass after full implementation

### Phase 2: Database Schema & Storage (T011-T020)
**Goal**: Implement SQLite database operations and schema management

**T011**: Implement database schema creation SQL
- **File**: `src/history/migrations.rs`
- **Acceptance**: Creates command_history table with all required columns
- **Dependencies**: T007 (migration system)
- **Test**: Schema creates successfully, all indexes present

**T012**: Implement FTS5 search table creation
- **File**: `src/history/migrations.rs`  
- **Acceptance**: command_search virtual table links to command_history
- **Dependencies**: T011 (base schema)
- **Test**: FTS5 table creates successfully, search indexing works

**T013**: Implement database connection management
- **File**: `src/history/storage.rs`
- **Acceptance**: Connection pooling, lazy initialization, error handling
- **Dependencies**: T006 (dependencies)
- **Test**: Connections open/close correctly, pool management works

**T014**: Implement SqliteHistoryManager struct
- **File**: `src/history/storage.rs`
- **Acceptance**: Implements HistoryManager trait with SQLite backend
- **Dependencies**: T005 (HistoryManager trait), T013 (connections)
- **Test**: Struct instantiates correctly, basic operations compile

**T015**: Implement store_command database operation
- **File**: `src/history/storage.rs`
- **Acceptance**: Inserts CommandHistoryEntry into database with proper binding
- **Dependencies**: T014 (SqliteHistoryManager), T002 (models)
- **Test**: Commands store successfully, auto-increment ID returned

**T016**: Implement get_recent_commands database operation
- **File**: `src/history/storage.rs`
- **Acceptance**: Retrieves latest N commands with proper ordering
- **Dependencies**: T015 (store operation)
- **Test**: Recent commands retrieved in correct timestamp order

**T017**: Implement get_command_by_id database operation
- **File**: `src/history/storage.rs`
- **Acceptance**: Retrieves single command by ID with None for missing
- **Dependencies**: T015 (store operation)
- **Test**: Existing commands found, non-existent return None

**T018**: Implement delete_command database operation
- **File**: `src/history/storage.rs`
- **Acceptance**: Removes command by ID, returns success boolean
- **Dependencies**: T015 (store operation)
- **Test**: Commands delete successfully, cascade to FTS5 table

**T019**: Write unit tests for all database operations [P]
- **File**: `tests/unit/test_sqlite_storage.rs`
- **Acceptance**: Test all CRUD operations with real SQLite database
- **Dependencies**: T015-T018 (database operations)
- **Test**: All database operations work correctly with edge cases

**T020**: Implement database migration runner and version tracking
- **File**: `src/history/migrations.rs`
- **Acceptance**: Applies migrations automatically, tracks schema versions
- **Dependencies**: T011-T012 (schema definitions)
- **Test**: Migrations apply correctly, version tracking persists

### Phase 3: Search Functionality (T021-T030)
**Goal**: Implement full-text search using SQLite FTS5

**T021**: Implement basic text search in SqliteHistoryManager [P]
- **File**: `src/history/storage.rs`
- **Acceptance**: search_commands method uses FTS5 for text matching
- **Dependencies**: T012 (FTS5 table), T014 (SqliteHistoryManager)
- **Test**: Text searches return relevant results, case-insensitive

**T022**: Implement search filters (date, backend, status) [P]
- **File**: `src/history/storage.rs`
- **Acceptance**: SearchFilters properly constrains query results
- **Dependencies**: T004 (SearchFilters), T021 (basic search)
- **Test**: All filter combinations work correctly

**T023**: Implement search result ranking and sorting [P]
- **File**: `src/history/search.rs`
- **Acceptance**: Results ranked by relevance and recency
- **Dependencies**: T021 (basic search)
- **Test**: Search results appear in expected relevance order

**T024**: Implement fuzzy search capabilities [P]
- **File**: `src/history/search.rs`
- **Acceptance**: Handles typos and partial matches gracefully
- **Dependencies**: T023 (ranking)
- **Test**: Fuzzy searches find intended commands despite typos

**T025**: Write performance tests for search operations [P]
- **File**: `tests/performance/test_search_benchmarks.rs`
- **Acceptance**: Search operations meet <50ms requirement with 10K entries
- **Dependencies**: T022 (search filters)
- **Test**: Benchmarks pass with test database of 10,000 entries

**T026**: Implement search indexing maintenance [P]
- **File**: `src/history/search.rs`
- **Acceptance**: FTS5 index rebuilds automatically when needed
- **Dependencies**: T021 (basic search)
- **Test**: Index rebuilds correctly after bulk operations

**T027**: Write unit tests for search functionality [P]
- **File**: `tests/unit/test_search_operations.rs`
- **Acceptance**: All search methods tested with comprehensive scenarios
- **Dependencies**: T024 (fuzzy search)
- **Test**: Search edge cases handled correctly

**T028**: Implement search result pagination
- **File**: `src/history/search.rs`
- **Acceptance**: Large result sets paginated efficiently
- **Dependencies**: T023 (ranking)
- **Test**: Pagination works correctly with consistent ordering

**T029**: Implement command categorization for search
- **File**: `src/history/models.rs`
- **Acceptance**: Commands automatically categorized by type (file, network, etc.)
- **Dependencies**: T002 (CommandHistoryEntry)
- **Test**: Categories assigned correctly for sample commands

**T030**: Write integration tests for search workflow
- **File**: `tests/integration/test_search_integration.rs`
- **Acceptance**: End-to-end search workflow with realistic data
- **Dependencies**: T024 (fuzzy search), T022 (filters)
- **Test**: Search integration works with complex query scenarios

### Phase 4: CLI Integration (T031-T040)
**Goal**: Add history commands to cmdai CLI interface

**T031**: Add history-related CLI arguments to clap configuration
- **File**: `src/cli/args.rs`
- **Acceptance**: --history, --search, --export flags added with help text
- **Dependencies**: None (parallel with storage development)
- **Test**: CLI help displays history options correctly

**T032**: Implement history display command handler
- **File**: `src/cli/history.rs`
- **Acceptance**: Displays recent commands with formatted output
- **Dependencies**: T031 (CLI args), T016 (get_recent_commands)
- **Test**: History display shows correct formatting and data

**T033**: Implement search command handler with interactive selection
- **File**: `src/cli/history.rs`
- **Acceptance**: Search results displayed with selection interface
- **Dependencies**: T032 (display), T022 (search filters)
- **Test**: Search interface allows command selection and execution

**T034**: Implement command replay functionality
- **File**: `src/cli/history.rs`
- **Acceptance**: Selected historical commands regenerated with current context
- **Dependencies**: T033 (selection), existing command generation pipeline
- **Test**: Replayed commands use current working directory and preferences

**T035**: Add history storage to main command generation workflow
- **File**: `src/main.rs`
- **Acceptance**: All generated commands automatically stored in history
- **Dependencies**: T015 (store_command), existing command generation
- **Test**: Commands appear in history immediately after generation

**T036**: Implement export command handler
- **File**: `src/cli/history.rs`
- **Acceptance**: Export history to JSON, CSV formats with --export flag
- **Dependencies**: T032 (display)
- **Test**: Export produces valid files in requested format

**T037**: Add history configuration options
- **File**: `src/config/mod.rs`
- **Acceptance**: History enabled/disabled, retention policy, privacy settings
- **Dependencies**: Existing config system
- **Test**: Configuration options properly parsed and applied

**T038**: Implement privacy filtering for sensitive data
- **File**: `src/history/privacy.rs`
- **Acceptance**: Automatically filters paths, tokens, sensitive arguments
- **Dependencies**: T002 (models), T037 (config)
- **Test**: Sensitive data properly removed from stored commands

**T039**: Write CLI integration tests [P]
- **File**: `tests/integration/test_cli_history.rs`
- **Acceptance**: Test all history CLI commands end-to-end
- **Dependencies**: T036 (export), T034 (replay)
- **Test**: All CLI history workflows function correctly

**T040**: Add history commands to shell completion
- **File**: `src/cli/completion.rs`
- **Acceptance**: Tab completion works for history flags and search terms
- **Dependencies**: T031 (CLI args)
- **Test**: Shell completion generates appropriate suggestions

### Phase 5: Performance & Polish (T041-T045)
**Goal**: Optimize performance and add production-ready features

**T041**: Implement connection pooling optimization
- **File**: `src/history/storage.rs`
- **Acceptance**: Database connections reused efficiently, no connection leaks
- **Dependencies**: T013 (connection management)
- **Test**: Connection pool handles concurrent operations correctly

**T042**: Implement automatic cleanup and retention policies
- **File**: `src/history/cleanup.rs`
- **Acceptance**: Old entries automatically purged based on policy configuration
- **Dependencies**: T037 (config), T018 (delete operation)
- **Test**: Cleanup runs automatically and respects user preferences

**T043**: Add comprehensive performance benchmarks
- **File**: `benches/history_performance.rs`
- **Acceptance**: Benchmark all operations against performance requirements
- **Dependencies**: All storage operations implemented
- **Test**: All benchmarks meet specified performance targets

**T044**: Implement error recovery and database repair
- **File**: `src/history/recovery.rs`
- **Acceptance**: Corrupted databases detected and repaired automatically
- **Dependencies**: T013 (connection management)
- **Test**: Recovery handles various corruption scenarios correctly

**T045**: Write comprehensive E2E tests for all history features
- **File**: `tests/e2e/test_history_complete.rs`
- **Acceptance**: Full workflow testing with realistic usage patterns
- **Dependencies**: All previous tasks completed
- **Test**: E2E tests cover all user scenarios from specification

---

## Task Execution Summary

### Task Count: 45 tasks total
- **Setup & Models**: 10 tasks (T001-T010)
- **Database & Storage**: 10 tasks (T011-T020)  
- **Search Functionality**: 10 tasks (T021-T030)
- **CLI Integration**: 10 tasks (T031-T040)
- **Performance & Polish**: 5 tasks (T041-T045)

### Parallel Execution Opportunities
- **[P] marked tasks**: Can execute simultaneously on different files
- **Models phase**: T001, T002, T003, T004, T005, T007, T008, T009 can run in parallel
- **Search phase**: T021, T022, T023, T024, T025, T026, T027 can run in parallel
- **Testing**: All test tasks can run parallel to implementation

### Critical Path Dependencies
1. **T002 → T008**: CommandHistoryEntry must exist before contract tests
2. **T005 → T009**: HistoryManager trait before trait tests
3. **T011 → T012**: Base schema before FTS5 table
4. **T015 → T016,T017,T018**: Store operation before other CRUD operations
5. **T022 → T025**: Search filters before performance tests

### Validation Gates
- **After T010**: All contract tests failing appropriately (TDD RED phase)
- **After T020**: Database operations functional (TDD GREEN phase)
- **After T030**: Search functionality complete with performance targets met
- **After T040**: CLI integration complete and user-testable
- **After T045**: Full feature complete and production-ready

---

*This task breakdown follows TDD methodology with contract tests first, ensures parallel execution where possible, and maintains strict dependencies to enable efficient implementation while meeting all constitutional requirements.*