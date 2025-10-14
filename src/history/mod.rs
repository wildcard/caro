//! Command history management module
//!
//! Provides persistent storage and retrieval of command history with rich metadata,
//! search capabilities, and integration with safety validation results.
//!
//! Inspired by Atuin's rich history management with privacy-first design.

pub mod models;
pub mod manager;
pub mod search;

pub use models::{
    CommandHistoryEntry, HistoryDatabase, HistoryQueryFilter, HistorySearchResult,
    ExecutionMetadata, SafetyMetadata,
};
pub use manager::HistoryManager;
pub use search::HistorySearch;