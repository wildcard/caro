//! Command history models and data structures
//!
//! Provides rich command history storage with metadata, search capabilities,
//! and privacy-preserving design inspired by Atuin.

use crate::models::{RiskLevel, SafetyLevel, ShellType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// A single command history entry with rich metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    /// Unique identifier for this history entry
    pub id: String,
    
    /// The actual shell command that was generated/executed
    pub command: String,
    
    /// Human-readable explanation of what the command does
    pub explanation: String,
    
    /// Shell type used for this command
    pub shell_type: ShellType,
    
    /// Working directory when command was executed
    pub working_directory: String,
    
    /// Timestamp when this entry was created
    pub timestamp: DateTime<Utc>,
    
    /// Original user input (natural language query)
    pub user_input: Option<String>,
    
    /// Execution-related metadata
    pub execution_metadata: Option<ExecutionMetadata>,
    
    /// Safety validation metadata
    pub safety_metadata: Option<SafetyMetadata>,
    
    /// Additional tags for categorization
    pub tags: Vec<String>,
    
    /// Session identifier for grouping related commands
    pub session_id: Option<String>,
    
    /// Hostname where command was executed
    pub hostname: Option<String>,
    
    /// Username who executed the command
    pub username: Option<String>,
}

impl CommandHistoryEntry {
    /// Create a new command history entry
    pub fn new(
        command: impl Into<String>,
        explanation: impl Into<String>,
        shell_type: ShellType,
        working_directory: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            command: command.into(),
            explanation: explanation.into(),
            shell_type,
            working_directory: working_directory.into(),
            timestamp: Utc::now(),
            user_input: None,
            execution_metadata: None,
            safety_metadata: None,
            tags: Vec::new(),
            session_id: None,
            hostname: None,
            username: None,
        }
    }
    
    /// Add user input (builder pattern)
    pub fn with_user_input(mut self, user_input: impl Into<String>) -> Self {
        self.user_input = Some(user_input.into());
        self
    }
    
    /// Add execution metadata (builder pattern)
    pub fn with_execution_metadata(mut self, metadata: ExecutionMetadata) -> Self {
        self.execution_metadata = Some(metadata);
        self
    }
    
    /// Add safety metadata (builder pattern)
    pub fn with_safety_metadata(mut self, metadata: SafetyMetadata) -> Self {
        self.safety_metadata = Some(metadata);
        self
    }
    
    /// Add tags (builder pattern)
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    /// Add session ID (builder pattern)
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
    
    /// Filter sensitive data from the entry
    pub fn filter_sensitive_data(mut self) -> Self {
        // Define patterns that might contain sensitive information
        let sensitive_patterns = [
            r"--key[=\s]+[^\s]+",
            r"--password[=\s]+[^\s]+",
            r"--token[=\s]+[^\s]+",
            r"--secret[=\s]+[^\s]+",
            r"[A-Z0-9]{20,}", // Potential API keys/secrets
        ];
        
        for pattern in &sensitive_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                self.command = regex.replace_all(&self.command, "[FILTERED]").to_string();
                if let Some(ref mut input) = self.user_input {
                    *input = regex.replace_all(input, "[FILTERED]").to_string();
                }
            }
        }
        
        self
    }
    
    /// Calculate relevance score for a search query
    pub fn calculate_relevance(&self, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let command_lower = self.command.to_lowercase();
        let explanation_lower = self.explanation.to_lowercase();
        
        let mut score: f64 = 0.0;
        
        // Exact substring match in command gets highest score
        if command_lower.contains(&query_lower) {
            score += 0.8;
        }
        
        // Exact substring match in explanation gets medium score
        if explanation_lower.contains(&query_lower) {
            score += 0.4;
        }
        
        // Word matches get additional points
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let command_words: Vec<&str> = command_lower.split_whitespace().collect();
        
        for query_word in &query_words {
            for command_word in &command_words {
                if command_word.contains(query_word) {
                    score += 0.1;
                }
            }
        }
        
        // Cap the score at 1.0
        score.min(1.0)
    }
}

/// Metadata about command execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Exit code of the command (if executed)
    pub exit_code: Option<i32>,
    
    /// Duration of command execution
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Backend used for command generation
    pub backend_used: String,
    
    /// Time taken to generate the command (milliseconds)
    pub generation_time_ms: u64,
    
    /// Name of the model used
    pub model_name: String,
    
    /// Confidence score from the generation
    pub confidence_score: f64,
}

/// Metadata about safety validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyMetadata {
    /// Risk level assessed by safety validator
    pub risk_level: RiskLevel,
    
    /// Safety level setting at time of validation
    pub safety_level: SafetyLevel,
    
    /// Time taken for safety validation (milliseconds)
    pub validation_time_ms: u64,
    
    /// Patterns that were matched during validation
    pub patterns_matched: Vec<String>,
    
    /// Whether user confirmed a risky command
    pub user_confirmed: bool,
}

/// Query filter for searching command history
#[derive(Debug, Clone, Default)]
pub struct HistoryQueryFilter {
    /// Pattern to match in command text
    pub command_pattern: Option<String>,
    
    /// Filter by shell type
    pub shell_type: Option<ShellType>,
    
    /// Filter by working directory
    pub working_directory: Option<String>,
    
    /// Start time for date range filter
    pub start_time: Option<DateTime<Utc>>,
    
    /// End time for date range filter
    pub end_time: Option<DateTime<Utc>>,
    
    /// Maximum risk level to include
    pub max_risk_level: Option<RiskLevel>,
    
    /// Filter by session ID
    pub session_id: Option<String>,
    
    /// Filter by hostname
    pub hostname: Option<String>,
    
    /// Filter by username
    pub username: Option<String>,
    
    /// Filter by tags
    pub tags: Vec<String>,
    
    /// Limit number of results
    pub limit: usize,
    
    /// Offset for pagination
    pub offset: usize,
}

impl HistoryQueryFilter {
    /// Create a new query filter
    pub fn new() -> Self {
        Self {
            limit: 100, // Default limit
            ..Default::default()
        }
    }
    
    /// Add command pattern filter (builder pattern)
    pub fn with_command_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.command_pattern = Some(pattern.into());
        self
    }
    
    /// Add shell type filter (builder pattern)
    pub fn with_shell_type(mut self, shell_type: ShellType) -> Self {
        self.shell_type = Some(shell_type);
        self
    }
    
    /// Add working directory filter (builder pattern)
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }
    
    /// Add time range filter (builder pattern)
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }
    
    /// Add maximum risk level filter (builder pattern)
    pub fn with_risk_level_max(mut self, max_risk: RiskLevel) -> Self {
        self.max_risk_level = Some(max_risk);
        self
    }
    
    /// Add session ID filter (builder pattern)
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
    
    /// Set result limit (builder pattern)
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    
    /// Set result offset (builder pattern)
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

/// Search result with relevance scoring
#[derive(Debug, Clone)]
pub struct HistorySearchResult {
    /// The matching history entry
    pub entry: CommandHistoryEntry,
    
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    
    /// Highlighted snippets showing matches
    pub highlights: Vec<String>,
}

/// Database connection and schema management
pub struct HistoryDatabase {
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}

impl HistoryDatabase {
    /// Create a new database connection
    pub fn new(database_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(database_path);
        let pool = r2d2::Pool::new(manager)?;
        
        let db = Self { pool };
        db.initialize_schema()?;
        Ok(db)
    }
    
    /// Initialize database schema
    fn initialize_schema(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS command_history (
                id TEXT PRIMARY KEY,
                command TEXT NOT NULL,
                explanation TEXT NOT NULL,
                shell_type TEXT NOT NULL,
                working_directory TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                user_input TEXT,
                execution_metadata TEXT,
                safety_metadata TEXT,
                tags TEXT,
                session_id TEXT,
                hostname TEXT,
                username TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )?;
        
        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_command_timestamp ON command_history(timestamp)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_command_shell ON command_history(shell_type)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_command_directory ON command_history(working_directory)",
            [],
        )?;
        
        Ok(())
    }
    
    /// Get database connection from pool
    pub fn get_connection(&self) -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>, Box<dyn std::error::Error>> {
        Ok(self.pool.get()?)
    }
}

// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}