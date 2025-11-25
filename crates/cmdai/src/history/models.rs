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

    /// Embedding vector for semantic search (384 dimensions for SentenceT5-Base)
    pub embedding_vector: Option<Vec<f32>>,

    /// Relevance score for search query ranking
    pub relevance_score: Option<f64>,
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
            embedding_vector: None,
            relevance_score: None,
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

/// Metadata about command execution and generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Exit code of the command (if executed)
    pub exit_code: Option<i32>,

    /// Wall clock time for command execution
    pub execution_time: Option<Duration>,

    /// Bytes of output produced by the command
    pub output_size: Option<usize>,

    /// Backend used for command generation (mlx, ollama, vllm, etc.)
    pub backend_used: String,

    /// AI inference duration
    #[serde(with = "duration_serde")]
    pub generation_time: Duration,

    /// Safety check duration
    #[serde(with = "duration_serde")]
    pub validation_time: Duration,

    /// Name of the model used
    pub model_name: String,

    /// Confidence score from the generation (0.0-1.0)
    pub confidence_score: f64,
}

/// Metadata about safety validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyMetadata {
    /// Risk level assessed by safety validator (Safe, Moderate, High, Critical)
    pub risk_level: RiskLevel,

    /// Triggered safety patterns that were matched
    pub patterns_matched: Vec<String>,

    /// Whether user approved risky command
    pub user_confirmed: bool,

    /// Numerical risk assessment score (0.0-1.0)
    pub safety_score: f64,

    /// Safety level setting at time of validation
    pub safety_level: SafetyLevel,

    /// Time taken for safety validation (milliseconds) - deprecated, use ExecutionMetadata::validation_time
    #[deprecated(note = "Use ExecutionMetadata::validation_time instead")]
    pub validation_time_ms: u64,
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
    pub fn get_connection(
        &self,
    ) -> Result<
        r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>,
        Box<dyn std::error::Error>,
    > {
        Ok(self.pool.get()?)
    }
}

impl ExecutionMetadata {
    /// Create new execution metadata with required fields
    pub fn new(
        backend_used: impl Into<String>,
        generation_time: Duration,
        validation_time: Duration,
        model_name: impl Into<String>,
        confidence_score: f64,
    ) -> Self {
        Self {
            exit_code: None,
            execution_time: None,
            output_size: None,
            backend_used: backend_used.into(),
            generation_time,
            validation_time,
            model_name: model_name.into(),
            confidence_score: confidence_score.clamp(0.0, 1.0),
        }
    }

    /// Set execution result (builder pattern)
    pub fn with_execution_result(
        mut self,
        exit_code: i32,
        execution_time: Duration,
        output_size: usize,
    ) -> Self {
        self.exit_code = Some(exit_code);
        self.execution_time = Some(execution_time);
        self.output_size = Some(output_size);
        self
    }

    /// Get total processing time (generation + validation)
    pub fn total_processing_time(&self) -> Duration {
        self.generation_time + self.validation_time
    }

    /// Check if command was executed successfully
    pub fn is_successful(&self) -> Option<bool> {
        self.exit_code.map(|code| code == 0)
    }

    /// Validate metadata constraints
    pub fn validate(&self) -> Result<(), String> {
        if self.confidence_score < 0.0 || self.confidence_score > 1.0 {
            return Err(format!(
                "Confidence score must be between 0.0 and 1.0, got {}",
                self.confidence_score
            ));
        }

        if self.backend_used.trim().is_empty() {
            return Err("Backend name cannot be empty".to_string());
        }

        if self.model_name.trim().is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        Ok(())
    }
}

impl SafetyMetadata {
    /// Create new safety metadata with required fields
    pub fn new(risk_level: RiskLevel, safety_score: f64, safety_level: SafetyLevel) -> Self {
        Self {
            risk_level,
            patterns_matched: Vec::new(),
            user_confirmed: false,
            safety_score: safety_score.clamp(0.0, 1.0),
            safety_level,
            #[allow(deprecated)]
            validation_time_ms: 0, // Deprecated field
        }
    }

    /// Add matched safety patterns (builder pattern)
    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns_matched = patterns;
        self
    }

    /// Mark as user confirmed (builder pattern)
    pub fn with_user_confirmation(mut self, confirmed: bool) -> Self {
        self.user_confirmed = confirmed;
        self
    }

    /// Check if command is considered safe
    pub fn is_safe(&self) -> bool {
        matches!(self.risk_level, RiskLevel::Safe) && self.safety_score < 0.3
    }

    /// Check if user confirmation is required
    pub fn requires_confirmation(&self) -> bool {
        matches!(self.risk_level, RiskLevel::High | RiskLevel::Critical) || self.safety_score > 0.7
    }

    /// Get risk description based on score and level
    pub fn get_risk_description(&self) -> String {
        match self.risk_level {
            RiskLevel::Low => format!("Low risk (score: {:.2})", self.safety_score),
            RiskLevel::Safe => format!("Safe command (score: {:.2})", self.safety_score),
            RiskLevel::Medium => format!("Medium risk (score: {:.2})", self.safety_score),
            RiskLevel::Moderate => format!("Moderate risk (score: {:.2})", self.safety_score),
            RiskLevel::High => format!("High risk command (score: {:.2})", self.safety_score),
            RiskLevel::Critical => format!(
                "Critical risk - dangerous command (score: {:.2})",
                self.safety_score
            ),
        }
    }

    /// Validate metadata constraints
    pub fn validate(&self) -> Result<(), String> {
        if self.safety_score < 0.0 || self.safety_score > 1.0 {
            return Err(format!(
                "Safety score must be between 0.0 and 1.0, got {}",
                self.safety_score
            ));
        }

        // Validate consistency between risk level and safety score
        let expected_range = match self.risk_level {
            RiskLevel::Low => (0.0, 0.2),
            RiskLevel::Safe => (0.0, 0.3),
            RiskLevel::Medium => (0.3, 0.5),
            RiskLevel::Moderate => (0.5, 0.7),
            RiskLevel::High => (0.7, 0.85),
            RiskLevel::Critical => (0.85, 1.0),
        };

        if self.safety_score < expected_range.0 || self.safety_score > expected_range.1 {
            return Err(format!(
                "Safety score {:.2} inconsistent with risk level {:?} (expected range: {:.2}-{:.2})",
                self.safety_score, self.risk_level, expected_range.0, expected_range.1
            ));
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_execution_metadata_creation() {
        let metadata = ExecutionMetadata::new(
            "mlx",
            Duration::from_millis(1500),
            Duration::from_millis(50),
            "Llama-3.2-3B-Instruct",
            0.95,
        );

        assert_eq!(metadata.backend_used, "mlx");
        assert_eq!(metadata.generation_time, Duration::from_millis(1500));
        assert_eq!(metadata.validation_time, Duration::from_millis(50));
        assert_eq!(metadata.model_name, "Llama-3.2-3B-Instruct");
        assert_eq!(metadata.confidence_score, 0.95);
        assert!(metadata.exit_code.is_none());
        assert!(metadata.execution_time.is_none());
        assert!(metadata.output_size.is_none());
    }

    #[test]
    fn test_execution_metadata_with_result() {
        let metadata = ExecutionMetadata::new(
            "ollama",
            Duration::from_millis(2000),
            Duration::from_millis(30),
            "llama3.2:3b",
            0.88,
        )
        .with_execution_result(0, Duration::from_millis(250), 1024);

        assert_eq!(metadata.exit_code, Some(0));
        assert_eq!(metadata.execution_time, Some(Duration::from_millis(250)));
        assert_eq!(metadata.output_size, Some(1024));
        assert_eq!(metadata.is_successful(), Some(true));
    }

    #[test]
    fn test_execution_metadata_total_processing_time() {
        let metadata = ExecutionMetadata::new(
            "mlx",
            Duration::from_millis(1500),
            Duration::from_millis(50),
            "model",
            0.9,
        );

        assert_eq!(
            metadata.total_processing_time(),
            Duration::from_millis(1550)
        );
    }

    #[test]
    fn test_execution_metadata_validation() {
        // Valid metadata
        let valid_metadata = ExecutionMetadata::new(
            "mlx",
            Duration::from_millis(1000),
            Duration::from_millis(50),
            "model",
            0.85,
        );
        assert!(valid_metadata.validate().is_ok());

        // Invalid confidence score
        let mut invalid_metadata = valid_metadata.clone();
        invalid_metadata.confidence_score = 1.5;
        assert!(invalid_metadata.validate().is_err());

        // Empty backend name
        let mut invalid_metadata = valid_metadata.clone();
        invalid_metadata.backend_used = "".to_string();
        assert!(invalid_metadata.validate().is_err());

        // Empty model name
        let mut invalid_metadata = valid_metadata;
        invalid_metadata.model_name = "".to_string();
        assert!(invalid_metadata.validate().is_err());
    }

    #[test]
    fn test_safety_metadata_creation() {
        let metadata = SafetyMetadata::new(RiskLevel::Moderate, 0.45, SafetyLevel::Moderate);

        assert_eq!(metadata.risk_level, RiskLevel::Moderate);
        assert_eq!(metadata.safety_score, 0.45);
        assert_eq!(metadata.safety_level, SafetyLevel::Moderate);
        assert!(metadata.patterns_matched.is_empty());
        assert!(!metadata.user_confirmed);
    }

    #[test]
    fn test_safety_metadata_with_patterns() {
        let metadata = SafetyMetadata::new(RiskLevel::High, 0.75, SafetyLevel::Strict)
            .with_patterns(vec!["rm -rf".to_string(), "sudo".to_string()])
            .with_user_confirmation(true);

        assert_eq!(metadata.patterns_matched.len(), 2);
        assert!(metadata.patterns_matched.contains(&"rm -rf".to_string()));
        assert!(metadata.user_confirmed);
    }

    #[test]
    fn test_safety_metadata_risk_assessment() {
        let safe_metadata = SafetyMetadata::new(RiskLevel::Safe, 0.15, SafetyLevel::Moderate);
        assert!(safe_metadata.is_safe());
        assert!(!safe_metadata.requires_confirmation());

        let high_risk_metadata = SafetyMetadata::new(RiskLevel::High, 0.75, SafetyLevel::Moderate);
        assert!(!high_risk_metadata.is_safe());
        assert!(high_risk_metadata.requires_confirmation());

        let critical_metadata = SafetyMetadata::new(RiskLevel::Critical, 0.95, SafetyLevel::Strict);
        assert!(!critical_metadata.is_safe());
        assert!(critical_metadata.requires_confirmation());
    }

    #[test]
    fn test_safety_metadata_descriptions() {
        let safe_metadata = SafetyMetadata::new(RiskLevel::Safe, 0.15, SafetyLevel::Moderate);
        assert!(safe_metadata
            .get_risk_description()
            .contains("Safe command"));

        let critical_metadata = SafetyMetadata::new(RiskLevel::Critical, 0.95, SafetyLevel::Strict);
        assert!(critical_metadata
            .get_risk_description()
            .contains("Critical risk"));
    }

    #[test]
    fn test_safety_metadata_validation() {
        // Valid metadata
        let valid_metadata = SafetyMetadata::new(RiskLevel::Moderate, 0.6, SafetyLevel::Moderate);
        assert!(valid_metadata.validate().is_ok());

        // Invalid safety score
        let mut invalid_metadata = valid_metadata.clone();
        invalid_metadata.safety_score = -0.1;
        assert!(invalid_metadata.validate().is_err());

        // Inconsistent risk level and score
        let mut invalid_metadata = valid_metadata;
        invalid_metadata.risk_level = RiskLevel::Critical;
        invalid_metadata.safety_score = 0.2; // Too low for Critical
        assert!(invalid_metadata.validate().is_err());
    }

    #[test]
    fn test_command_history_entry_with_extended_metadata() {
        let execution_metadata = ExecutionMetadata::new(
            "mlx",
            Duration::from_millis(1200),
            Duration::from_millis(45),
            "Llama-3.2-3B",
            0.92,
        )
        .with_execution_result(0, Duration::from_millis(100), 512);

        let safety_metadata = SafetyMetadata::new(RiskLevel::Safe, 0.15, SafetyLevel::Moderate);

        let entry = CommandHistoryEntry::new(
            "ls -la",
            "List all files in current directory",
            ShellType::Bash,
            "/home/user",
        )
        .with_execution_metadata(execution_metadata)
        .with_safety_metadata(safety_metadata);

        assert!(entry.execution_metadata.is_some());
        assert!(entry.safety_metadata.is_some());

        let exec_meta = entry.execution_metadata.unwrap();
        assert_eq!(exec_meta.backend_used, "mlx");
        assert_eq!(exec_meta.confidence_score, 0.92);
        assert_eq!(exec_meta.is_successful(), Some(true));

        let safety_meta = entry.safety_metadata.unwrap();
        assert_eq!(safety_meta.risk_level, RiskLevel::Safe);
        assert!(safety_meta.is_safe());
    }
}
