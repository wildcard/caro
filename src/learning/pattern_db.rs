//! Pattern database for storing command interaction history
//!
//! The pattern database stores all command interactions locally in a SQLite database.
//! It supports:
//! - Recording command generations with full context
//! - Learning from user edits
//! - Querying historical patterns
//! - Privacy-preserving local storage

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Command pattern stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    pub id: Uuid,
    pub user_prompt: String,
    pub generated_command: String,
    pub final_command: Option<String>,
    pub context_snapshot: serde_json::Value,
    pub execution_success: Option<bool>,
    pub user_rating: Option<u8>,
    pub timestamp: DateTime<Utc>,
}

/// Pattern database manager
#[derive(Clone)]
pub struct PatternDB {
    pub(crate) pool: Arc<SqlitePool>,
    db_path: PathBuf,
}

impl PatternDB {
    /// Create a new pattern database
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists (except for special paths)
        if db_path.to_str() != Some(":memory:") {
            if let Some(parent) = db_path.parent() {
                if !parent.as_os_str().is_empty() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .context("Failed to create database directory")?;
                }
            }
        }

        // Create connection pool
        let connection_string = if db_path.to_str() == Some(":memory:") {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite://{}", db_path.display())
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .context("Failed to connect to SQLite database")?;

        let db = Self {
            pool: Arc::new(pool),
            db_path,
        };

        // Initialize schema
        db.initialize_schema().await?;

        Ok(db)
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        // Create command_patterns table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS command_patterns (
                id TEXT PRIMARY KEY,
                user_prompt TEXT NOT NULL,
                generated_command TEXT NOT NULL,
                final_command TEXT,
                context_snapshot TEXT NOT NULL,
                execution_success INTEGER,
                user_rating INTEGER,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .context("Failed to create command_patterns table")?;

        // Create indices for performance
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_timestamp
            ON command_patterns(timestamp DESC)
            "#,
        )
        .execute(self.pool.as_ref())
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_prompt
            ON command_patterns(user_prompt)
            "#,
        )
        .execute(self.pool.as_ref())
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_edited
            ON command_patterns(final_command)
            WHERE final_command IS NOT NULL
            "#,
        )
        .execute(self.pool.as_ref())
        .await?;

        // Create improvement_patterns table for learned improvements
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS improvement_patterns (
                id TEXT PRIMARY KEY,
                original_template TEXT NOT NULL,
                improvement_template TEXT NOT NULL,
                frequency INTEGER NOT NULL DEFAULT 1,
                contexts TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .context("Failed to create improvement_patterns table")?;

        Ok(())
    }

    /// Record a command interaction
    pub async fn record_interaction(&self, pattern: CommandPattern) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO command_patterns
            (id, user_prompt, generated_command, final_command, context_snapshot,
             execution_success, user_rating, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(pattern.id.to_string())
        .bind(&pattern.user_prompt)
        .bind(&pattern.generated_command)
        .bind(&pattern.final_command)
        .bind(serde_json::to_string(&pattern.context_snapshot)?)
        .bind(pattern.execution_success)
        .bind(pattern.user_rating.map(|r| r as i32))
        .bind(pattern.timestamp.to_rfc3339())
        .execute(self.pool.as_ref())
        .await
        .context("Failed to insert command pattern")?;

        Ok(())
    }

    /// Update pattern when user edits the command
    pub async fn learn_from_edit(&self, pattern_id: Uuid, edited_command: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE command_patterns
            SET final_command = ?
            WHERE id = ?
            "#,
        )
        .bind(edited_command)
        .bind(pattern_id.to_string())
        .execute(self.pool.as_ref())
        .await
        .context("Failed to update pattern with edited command")?;

        Ok(())
    }

    /// Get a pattern by ID
    pub async fn get_pattern_by_id(&self, pattern_id: Uuid) -> Result<Option<CommandPattern>> {
        let result = sqlx::query_as::<_, PatternRow>(
            r#"
            SELECT id, user_prompt, generated_command, final_command,
                   context_snapshot, execution_success, user_rating, timestamp
            FROM command_patterns
            WHERE id = ?
            "#,
        )
        .bind(pattern_id.to_string())
        .fetch_optional(self.pool.as_ref())
        .await?;

        result.map(|row| row.to_pattern()).transpose()
    }

    /// Find patterns by prompt (exact or partial match)
    pub async fn find_by_prompt(&self, prompt: &str, limit: usize) -> Result<Vec<CommandPattern>> {
        let rows = sqlx::query_as::<_, PatternRow>(
            r#"
            SELECT id, user_prompt, generated_command, final_command,
                   context_snapshot, execution_success, user_rating, timestamp
            FROM command_patterns
            WHERE user_prompt LIKE ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(format!("%{}%", prompt))
        .bind(limit as i64)
        .fetch_all(self.pool.as_ref())
        .await?;

        rows.into_iter()
            .map(|row| row.to_pattern())
            .collect::<Result<Vec<_>>>()
    }

    /// Get user patterns from the last N days
    pub async fn get_user_patterns(&self, days: u32) -> Result<Vec<CommandPattern>> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let rows = sqlx::query_as::<_, PatternRow>(
            r#"
            SELECT id, user_prompt, generated_command, final_command,
                   context_snapshot, execution_success, user_rating, timestamp
            FROM command_patterns
            WHERE timestamp >= ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(cutoff.to_rfc3339())
        .fetch_all(self.pool.as_ref())
        .await?;

        rows.into_iter()
            .map(|row| row.to_pattern())
            .collect::<Result<Vec<_>>>()
    }

    /// Get patterns that were edited by the user
    pub async fn get_edited_patterns(&self, limit: usize) -> Result<Vec<CommandPattern>> {
        let rows = sqlx::query_as::<_, PatternRow>(
            r#"
            SELECT id, user_prompt, generated_command, final_command,
                   context_snapshot, execution_success, user_rating, timestamp
            FROM command_patterns
            WHERE final_command IS NOT NULL
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit as i64)
        .fetch_all(self.pool.as_ref())
        .await?;

        rows.into_iter()
            .map(|row| row.to_pattern())
            .collect::<Result<Vec<_>>>()
    }

    /// Count total patterns in database
    pub async fn count_patterns(&self) -> Result<usize> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM command_patterns")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok(count.0 as usize)
    }

    /// Count patterns that were edited
    pub async fn count_edited_patterns(&self) -> Result<usize> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM command_patterns WHERE final_command IS NOT NULL",
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(count.0 as usize)
    }

    /// Get all patterns (for similarity search)
    pub async fn get_all_patterns(&self) -> Result<Vec<CommandPattern>> {
        let rows = sqlx::query_as::<_, PatternRow>(
            r#"
            SELECT id, user_prompt, generated_command, final_command,
                   context_snapshot, execution_success, user_rating, timestamp
            FROM command_patterns
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        rows.into_iter()
            .map(|row| row.to_pattern())
            .collect::<Result<Vec<_>>>()
    }

    /// Clear all patterns (privacy feature)
    pub async fn clear_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM command_patterns")
            .execute(self.pool.as_ref())
            .await?;

        sqlx::query("DELETE FROM improvement_patterns")
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    /// Store an improvement pattern
    pub async fn store_improvement_pattern(
        &self,
        original: &str,
        improvement: &str,
        contexts: &[String],
    ) -> Result<()> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let contexts_json = serde_json::to_string(contexts)?;

        // Try to update existing pattern first
        let result = sqlx::query(
            r#"
            UPDATE improvement_patterns
            SET frequency = frequency + 1, updated_at = ?
            WHERE original_template = ? AND improvement_template = ?
            "#,
        )
        .bind(&now)
        .bind(original)
        .bind(improvement)
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            // Insert new pattern
            sqlx::query(
                r#"
                INSERT INTO improvement_patterns
                (id, original_template, improvement_template, frequency, contexts, created_at, updated_at)
                VALUES (?, ?, ?, 1, ?, ?, ?)
                "#,
            )
            .bind(id.to_string())
            .bind(original)
            .bind(improvement)
            .bind(contexts_json)
            .bind(&now)
            .bind(&now)
            .execute(self.pool.as_ref())
            .await?;
        }

        Ok(())
    }

    /// Get improvement patterns
    pub async fn get_improvement_patterns(&self) -> Result<Vec<ImprovementPatternRow>> {
        let patterns = sqlx::query_as::<_, ImprovementPatternRow>(
            r#"
            SELECT original_template, improvement_template, frequency, contexts
            FROM improvement_patterns
            ORDER BY frequency DESC
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(patterns)
    }
}

/// Database row for command patterns
#[derive(sqlx::FromRow)]
struct PatternRow {
    id: String,
    user_prompt: String,
    generated_command: String,
    final_command: Option<String>,
    context_snapshot: String,
    execution_success: Option<i32>,
    user_rating: Option<i32>,
    timestamp: String,
}

impl PatternRow {
    fn to_pattern(self) -> Result<CommandPattern> {
        Ok(CommandPattern {
            id: Uuid::parse_str(&self.id)?,
            user_prompt: self.user_prompt,
            generated_command: self.generated_command,
            final_command: self.final_command,
            context_snapshot: serde_json::from_str(&self.context_snapshot)?,
            execution_success: self.execution_success.map(|v| v != 0),
            user_rating: self.user_rating.map(|v| v as u8),
            timestamp: DateTime::parse_from_rfc3339(&self.timestamp)?.with_timezone(&Utc),
        })
    }
}

/// Database row for improvement patterns
#[derive(sqlx::FromRow)]
pub struct ImprovementPatternRow {
    pub original_template: String,
    pub improvement_template: String,
    pub frequency: i64,
    pub contexts: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_pattern_db_create() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();
        assert_eq!(db.count_patterns().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_record_and_retrieve_pattern() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();

        let pattern = CommandPattern {
            id: Uuid::new_v4(),
            user_prompt: "list files".to_string(),
            generated_command: "ls -la".to_string(),
            final_command: None,
            context_snapshot: serde_json::json!({}),
            execution_success: Some(true),
            user_rating: Some(5),
            timestamp: Utc::now(),
        };

        let pattern_id = pattern.id;
        db.record_interaction(pattern).await.unwrap();

        let retrieved = db.get_pattern_by_id(pattern_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_prompt, "list files");
    }

    #[tokio::test]
    async fn test_learn_from_edit() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();

        let pattern = CommandPattern {
            id: Uuid::new_v4(),
            user_prompt: "find logs".to_string(),
            generated_command: "find . -name '*.log'".to_string(),
            final_command: None,
            context_snapshot: serde_json::json!({}),
            execution_success: None,
            user_rating: None,
            timestamp: Utc::now(),
        };

        let pattern_id = pattern.id;
        db.record_interaction(pattern).await.unwrap();

        db.learn_from_edit(pattern_id, "find . -name '*.log' -type f")
            .await
            .unwrap();

        let retrieved = db.get_pattern_by_id(pattern_id).await.unwrap().unwrap();
        assert_eq!(
            retrieved.final_command,
            Some("find . -name '*.log' -type f".to_string())
        );
    }

    #[tokio::test]
    async fn test_count_patterns() {
        let db = PatternDB::new(PathBuf::from(":memory:")).await.unwrap();

        for i in 0..5 {
            let pattern = CommandPattern {
                id: Uuid::new_v4(),
                user_prompt: format!("prompt {}", i),
                generated_command: format!("cmd {}", i),
                final_command: None,
                context_snapshot: serde_json::json!({}),
                execution_success: None,
                user_rating: None,
                timestamp: Utc::now(),
            };
            db.record_interaction(pattern).await.unwrap();
        }

        assert_eq!(db.count_patterns().await.unwrap(), 5);
    }
}
