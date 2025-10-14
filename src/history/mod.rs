//! Command history management module
//!
//! Provides persistent storage and retrieval of command history with rich metadata,
//! search capabilities, and integration with safety validation results.
//!
//! Inspired by Atuin's rich history management with privacy-first design.

pub mod models;
pub mod manager;
pub mod migrations;
pub mod search;

pub use models::{
    CommandHistoryEntry, HistoryDatabase, HistoryQueryFilter, HistorySearchResult,
    ExecutionMetadata, SafetyMetadata,
};
pub use manager::{HistoryManager, PaginatedHistory, RetentionPolicy, RetentionStats};
pub use migrations::{
    MigrationManager, Migration, DatabaseFeature, MigrationResult, DatabaseState,
};
pub use search::{SearchQuery, SearchFilters, SearchResult, DateRange};