//! Command history management module
//!
//! Provides persistent storage and retrieval of command history with rich metadata,
//! search capabilities, and integration with safety validation results.
//!
//! Inspired by Atuin's rich history management with privacy-first design.

pub mod manager;
pub mod migrations;
pub mod models;
pub mod search;

pub use manager::{HistoryManager, PaginatedHistory, RetentionPolicy, RetentionStats};
pub use migrations::{
    DatabaseFeature, DatabaseState, Migration, MigrationManager, MigrationResult,
};
pub use models::{
    CommandHistoryEntry, ExecutionMetadata, HistoryDatabase, HistoryQueryFilter,
    HistorySearchResult, SafetyMetadata,
};
pub use search::{DateRange, SearchFilters, SearchQuery, SearchResult};
