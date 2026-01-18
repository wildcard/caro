//! Multi-collection schema for specialized knowledge storage
//!
//! Defines 5 specialized collections for different types of command knowledge:
//! - Commands: Successful command executions
//! - Corrections: Agentic loop refinements
//! - Docs: Indexed documentation (man, tldr, help)
//! - Preferences: User/profile-specific patterns
//! - Context: Project/repository-specific knowledge

use serde::{Deserialize, Serialize};
use std::fmt;

/// Collection types for specialized knowledge storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionType {
    /// Successful command executions from user interactions
    Commands,
    /// Command corrections from agentic refinement loops
    Corrections,
    /// Indexed documentation (man pages, tldr, help output, GitHub docs)
    Docs,
    /// User/profile-specific command preferences and patterns
    Preferences,
    /// Project/repository-specific context and conventions
    Context,
}

impl CollectionType {
    /// Get the collection name as used in the vector database
    pub fn name(&self) -> &'static str {
        match self {
            Self::Commands => "caro_commands",
            Self::Corrections => "caro_corrections",
            Self::Docs => "caro_command_docs",
            Self::Preferences => "caro_user_preferences",
            Self::Context => "caro_project_context",
        }
    }

    /// Get a human-readable description of the collection's purpose
    pub fn description(&self) -> &'static str {
        match self {
            Self::Commands => "Successful command executions from user interactions",
            Self::Corrections => "Command corrections from agentic refinement loops",
            Self::Docs => "Indexed documentation (man pages, tldr, help output, GitHub docs)",
            Self::Preferences => "User/profile-specific command preferences and patterns",
            Self::Context => "Project/repository-specific context and conventions",
        }
    }

    /// Get all collection types
    pub fn all() -> &'static [CollectionType] {
        &[
            Self::Commands,
            Self::Corrections,
            Self::Docs,
            Self::Preferences,
            Self::Context,
        ]
    }

    /// Parse a collection name into a CollectionType
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "caro_commands" => Some(Self::Commands),
            "caro_corrections" => Some(Self::Corrections),
            "caro_command_docs" => Some(Self::Docs),
            "caro_user_preferences" => Some(Self::Preferences),
            "caro_project_context" => Some(Self::Context),
            _ => None,
        }
    }

    /// Check if this collection stores user-generated content
    pub fn is_user_content(&self) -> bool {
        matches!(self, Self::Commands | Self::Corrections)
    }

    /// Check if this collection stores indexed external content
    pub fn is_indexed_content(&self) -> bool {
        matches!(self, Self::Docs)
    }

    /// Check if this collection stores personalization data
    pub fn is_personalization(&self) -> bool {
        matches!(self, Self::Preferences | Self::Context)
    }
}

impl fmt::Display for CollectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for CollectionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_name(s).ok_or_else(|| {
            format!(
                "Unknown collection type '{}'. Valid types: {}",
                s,
                Self::all()
                    .iter()
                    .map(|c| c.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
}

/// Collection metadata for management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    /// Type of collection
    pub collection_type: CollectionType,
    /// Number of entries in the collection
    pub entry_count: usize,
    /// Collection size in bytes (if available)
    pub size_bytes: Option<u64>,
    /// When the collection was created
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When the collection was last updated
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl CollectionInfo {
    /// Create basic collection info with just type and count
    pub fn new(collection_type: CollectionType, entry_count: usize) -> Self {
        Self {
            collection_type,
            entry_count,
            size_bytes: None,
            created_at: None,
            last_updated: None,
        }
    }

    /// Create collection info with full metadata
    pub fn with_metadata(
        collection_type: CollectionType,
        entry_count: usize,
        size_bytes: u64,
        created_at: chrono::DateTime<chrono::Utc>,
        last_updated: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            collection_type,
            entry_count,
            size_bytes: Some(size_bytes),
            created_at: Some(created_at),
            last_updated: Some(last_updated),
        }
    }
}

/// Query scope for searching collections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryScope {
    /// Search only a specific collection
    Single(CollectionType),
    /// Search all collections
    All,
    /// Search only user-generated content (Commands + Corrections)
    UserContent,
    /// Search only indexed documentation
    Documentation,
    /// Search only personalization data
    Personalization,
}

impl QueryScope {
    /// Get the collection types included in this scope
    pub fn collections(&self) -> Vec<CollectionType> {
        match self {
            Self::Single(ct) => vec![*ct],
            Self::All => CollectionType::all().to_vec(),
            Self::UserContent => vec![CollectionType::Commands, CollectionType::Corrections],
            Self::Documentation => vec![CollectionType::Docs],
            Self::Personalization => {
                vec![CollectionType::Preferences, CollectionType::Context]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_names() {
        assert_eq!(CollectionType::Commands.name(), "caro_commands");
        assert_eq!(CollectionType::Corrections.name(), "caro_corrections");
        assert_eq!(CollectionType::Docs.name(), "caro_command_docs");
        assert_eq!(CollectionType::Preferences.name(), "caro_user_preferences");
        assert_eq!(CollectionType::Context.name(), "caro_project_context");
    }

    #[test]
    fn test_collection_from_name() {
        assert_eq!(
            CollectionType::from_name("caro_commands"),
            Some(CollectionType::Commands)
        );
        assert_eq!(CollectionType::from_name("unknown"), None);
    }

    #[test]
    fn test_collection_types() {
        assert!(CollectionType::Commands.is_user_content());
        assert!(!CollectionType::Docs.is_user_content());
        assert!(CollectionType::Docs.is_indexed_content());
        assert!(CollectionType::Preferences.is_personalization());
    }

    #[test]
    fn test_query_scope() {
        assert_eq!(
            QueryScope::Single(CollectionType::Commands).collections(),
            vec![CollectionType::Commands]
        );
        assert_eq!(QueryScope::All.collections().len(), 5);
        assert_eq!(QueryScope::UserContent.collections().len(), 2);
    }

    #[test]
    fn test_collection_parsing() {
        assert_eq!(
            "caro_commands".parse::<CollectionType>().unwrap(),
            CollectionType::Commands
        );
        assert!("invalid".parse::<CollectionType>().is_err());
    }
}
