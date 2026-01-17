//! Local Feedback Storage
//!
//! This module provides SQLite-based local storage for feedback submissions.
//! It allows users to:
//!
//! - Store feedback locally before or after GitHub submission
//! - Track the status of submitted feedback
//! - Query historical feedback submissions
//! - Update feedback status when syncing with GitHub

use crate::feedback::types::*;
use crate::feedback::FeedbackError;
use rusqlite::{params, Connection};
use std::path::Path;

// =============================================================================
// Database Structure
// =============================================================================

/// Local SQLite database for feedback storage
pub struct FeedbackDatabase {
    conn: Connection,
}

impl FeedbackDatabase {
    /// Create a new FeedbackDatabase with the given data directory
    ///
    /// # Arguments
    /// * `data_dir` - Directory where the database file will be stored
    ///
    /// # Returns
    /// Result containing the database instance or an error
    pub fn new(data_dir: &Path) -> Result<Self, FeedbackError> {
        // Ensure directory exists
        std::fs::create_dir_all(data_dir).map_err(|e| {
            FeedbackError::StorageError(format!("Failed to create data directory: {}", e))
        })?;

        let db_path = data_dir.join("feedback.db");
        let conn = Connection::open(&db_path)?;

        let db = Self { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Create an in-memory database (useful for testing)
    #[cfg(test)]
    pub fn in_memory() -> Result<Self, FeedbackError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize the database schema
    fn initialize_schema(&self) -> Result<(), FeedbackError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS feedback (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                user_description TEXT NOT NULL,
                reproduction_steps TEXT,
                context_json TEXT NOT NULL,
                github_issue_url TEXT,
                status TEXT NOT NULL
            )",
            [],
        )?;

        // Create index on status for efficient filtering
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_feedback_status ON feedback(status)",
            [],
        )?;

        // Create index on timestamp for efficient ordering
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_feedback_timestamp ON feedback(timestamp)",
            [],
        )?;

        Ok(())
    }

    /// Save a new feedback submission to the database
    ///
    /// # Arguments
    /// * `feedback` - The feedback to save
    ///
    /// # Returns
    /// Result indicating success or an error
    pub fn save(&self, feedback: &Feedback) -> Result<(), FeedbackError> {
        let context_json = serde_json::to_string(&feedback.context)?;
        let status_str = feedback.status.to_string();

        self.conn.execute(
            "INSERT INTO feedback (
                id, timestamp, user_description, reproduction_steps,
                context_json, github_issue_url, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                feedback.id.as_str(),
                feedback.timestamp.to_rfc3339(),
                feedback.user_description,
                feedback.reproduction_steps,
                context_json,
                feedback.github_issue_url,
                status_str,
            ],
        )?;

        Ok(())
    }

    /// Get a feedback submission by ID
    ///
    /// # Arguments
    /// * `id` - The feedback ID to look up
    ///
    /// # Returns
    /// Result containing Some(Feedback) if found, None if not found, or an error
    pub fn get(&self, id: &FeedbackId) -> Result<Option<Feedback>, FeedbackError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, user_description, reproduction_steps,
                    context_json, github_issue_url, status
             FROM feedback WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.as_str()])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row_to_feedback(row)?))
        } else {
            Ok(None)
        }
    }

    /// Update an existing feedback submission
    ///
    /// # Arguments
    /// * `feedback` - The feedback with updated values
    ///
    /// # Returns
    /// Result indicating success or an error
    pub fn update(&self, feedback: &Feedback) -> Result<(), FeedbackError> {
        let context_json = serde_json::to_string(&feedback.context)?;
        let status_str = feedback.status.to_string();

        let rows_affected = self.conn.execute(
            "UPDATE feedback SET
                timestamp = ?2,
                user_description = ?3,
                reproduction_steps = ?4,
                context_json = ?5,
                github_issue_url = ?6,
                status = ?7
             WHERE id = ?1",
            params![
                feedback.id.as_str(),
                feedback.timestamp.to_rfc3339(),
                feedback.user_description,
                feedback.reproduction_steps,
                context_json,
                feedback.github_issue_url,
                status_str,
            ],
        )?;

        if rows_affected == 0 {
            return Err(FeedbackError::StorageError(format!(
                "Feedback {} not found",
                feedback.id
            )));
        }

        Ok(())
    }

    /// Delete a feedback submission
    ///
    /// # Arguments
    /// * `id` - The feedback ID to delete
    ///
    /// # Returns
    /// Result containing true if deleted, false if not found
    pub fn delete(&self, id: &FeedbackId) -> Result<bool, FeedbackError> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM feedback WHERE id = ?1", params![id.as_str()])?;

        Ok(rows_affected > 0)
    }

    /// List all feedback submissions
    ///
    /// # Returns
    /// Result containing a vector of all feedback submissions
    pub fn list_all(&self) -> Result<Vec<Feedback>, FeedbackError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, user_description, reproduction_steps,
                    context_json, github_issue_url, status
             FROM feedback ORDER BY timestamp DESC",
        )?;

        let mut rows = stmt.query([])?;
        let mut feedbacks = Vec::new();

        while let Some(row) = rows.next()? {
            feedbacks.push(row_to_feedback(row)?);
        }

        Ok(feedbacks)
    }

    /// List feedback submissions by status
    ///
    /// # Arguments
    /// * `status` - The status to filter by
    ///
    /// # Returns
    /// Result containing a vector of matching feedback submissions
    pub fn list_by_status(&self, status: FeedbackStatus) -> Result<Vec<Feedback>, FeedbackError> {
        let status_str = status.to_string();

        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, user_description, reproduction_steps,
                    context_json, github_issue_url, status
             FROM feedback WHERE status = ?1 ORDER BY timestamp DESC",
        )?;

        let mut rows = stmt.query(params![status_str])?;
        let mut feedbacks = Vec::new();

        while let Some(row) = rows.next()? {
            feedbacks.push(row_to_feedback(row)?);
        }

        Ok(feedbacks)
    }

    /// Get the count of feedback submissions by status
    ///
    /// # Returns
    /// Result containing a map of status to count
    pub fn count_by_status(&self) -> Result<std::collections::HashMap<String, usize>, FeedbackError>
    {
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM feedback GROUP BY status")?;

        let mut rows = stmt.query([])?;
        let mut counts = std::collections::HashMap::new();

        while let Some(row) = rows.next()? {
            let status: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            counts.insert(status, count as usize);
        }

        Ok(counts)
    }

    /// Update just the status and GitHub issue URL of a feedback
    ///
    /// This is a convenience method for updating status during sync operations.
    ///
    /// # Arguments
    /// * `id` - The feedback ID to update
    /// * `status` - The new status
    /// * `github_issue_url` - Optional GitHub issue URL
    pub fn update_status(
        &self,
        id: &FeedbackId,
        status: FeedbackStatus,
        github_issue_url: Option<&str>,
    ) -> Result<(), FeedbackError> {
        let status_str = status.to_string();

        let rows_affected = self.conn.execute(
            "UPDATE feedback SET status = ?2, github_issue_url = ?3 WHERE id = ?1",
            params![id.as_str(), status_str, github_issue_url],
        )?;

        if rows_affected == 0 {
            return Err(FeedbackError::StorageError(format!(
                "Feedback {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Get recent feedback (last N submissions)
    ///
    /// # Arguments
    /// * `limit` - Maximum number of submissions to return
    pub fn get_recent(&self, limit: usize) -> Result<Vec<Feedback>, FeedbackError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, user_description, reproduction_steps,
                    context_json, github_issue_url, status
             FROM feedback ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let mut rows = stmt.query(params![limit as i64])?;
        let mut feedbacks = Vec::new();

        while let Some(row) = rows.next()? {
            feedbacks.push(row_to_feedback(row)?);
        }

        Ok(feedbacks)
    }
}

/// Convert a database row to a Feedback struct
fn row_to_feedback(row: &rusqlite::Row) -> Result<Feedback, FeedbackError> {
    let id_str: String = row.get(0)?;
    let timestamp_str: String = row.get(1)?;
    let user_description: String = row.get(2)?;
    let reproduction_steps: Option<String> = row.get(3)?;
    let context_json: String = row.get(4)?;
    let github_issue_url: Option<String> = row.get(5)?;
    let status_str: String = row.get(6)?;

    let id = FeedbackId::parse(&id_str).map_err(|e| FeedbackError::StorageError(e.to_string()))?;

    let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
        .map_err(|e| FeedbackError::StorageError(format!("Invalid timestamp: {}", e)))?
        .with_timezone(&chrono::Utc);

    let context: FeedbackContext = serde_json::from_str(&context_json)?;

    let status: FeedbackStatus = status_str
        .parse()
        .map_err(|e: String| FeedbackError::StorageError(e))?;

    Ok(Feedback {
        id,
        timestamp,
        user_description,
        reproduction_steps,
        context,
        github_issue_url,
        status,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // =========================================================================
    // Database Initialization Tests
    // =========================================================================

    #[test]
    fn test_database_initialization() {
        let dir = tempdir().unwrap();
        let db = FeedbackDatabase::new(dir.path()).expect("Should create database");

        // Verify we can create it again (tables should exist)
        drop(db);
        let _db2 = FeedbackDatabase::new(dir.path()).expect("Should reopen database");
    }

    #[test]
    fn test_in_memory_database() {
        let db = FeedbackDatabase::in_memory().expect("Should create in-memory database");
        assert!(db.list_all().unwrap().is_empty());
    }

    // =========================================================================
    // Save and Retrieve Tests
    // =========================================================================

    #[test]
    fn test_save_and_retrieve_feedback() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let feedback = create_test_feedback();

        // Save
        db.save(&feedback).expect("Should save feedback");

        // Retrieve
        let retrieved = db.get(&feedback.id).expect("Should get feedback");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(feedback.id, retrieved.id);
        assert_eq!(feedback.user_description, retrieved.user_description);
        assert_eq!(feedback.status, retrieved.status);
    }

    #[test]
    fn test_get_nonexistent_feedback() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let id = FeedbackId::parse("fb-xyz123").unwrap();

        let result = db.get(&id).expect("Should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_save_duplicate_id_fails() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let feedback = create_test_feedback();

        db.save(&feedback).expect("First save should succeed");

        // Try to save again with same ID
        let result = db.save(&feedback);
        assert!(result.is_err(), "Duplicate save should fail");
    }

    // =========================================================================
    // Update Tests
    // =========================================================================

    #[test]
    fn test_update_feedback_status() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let mut feedback = create_test_feedback();

        db.save(&feedback).unwrap();

        // Update status
        feedback.status = FeedbackStatus::Resolved;
        feedback.github_issue_url = Some("https://github.com/test/repo/issues/1".to_string());
        db.update(&feedback).expect("Should update feedback");

        // Verify update
        let retrieved = db.get(&feedback.id).unwrap().unwrap();
        assert_eq!(FeedbackStatus::Resolved, retrieved.status);
        assert_eq!(
            Some("https://github.com/test/repo/issues/1".to_string()),
            retrieved.github_issue_url
        );
    }

    #[test]
    fn test_update_status_convenience_method() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let feedback = create_test_feedback();

        db.save(&feedback).unwrap();

        // Update just status
        db.update_status(
            &feedback.id,
            FeedbackStatus::InProgress,
            Some("https://github.com/test/repo/issues/2"),
        )
        .expect("Should update status");

        let retrieved = db.get(&feedback.id).unwrap().unwrap();
        assert_eq!(FeedbackStatus::InProgress, retrieved.status);
    }

    #[test]
    fn test_update_nonexistent_feedback() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let feedback = create_test_feedback();

        // Try to update without saving first
        let result = db.update(&feedback);
        assert!(result.is_err(), "Update of nonexistent should fail");
    }

    // =========================================================================
    // Delete Tests
    // =========================================================================

    #[test]
    fn test_delete_feedback() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let feedback = create_test_feedback();

        db.save(&feedback).unwrap();

        let deleted = db.delete(&feedback.id).expect("Should delete");
        assert!(deleted, "Should return true when deleted");

        // Verify deleted
        let retrieved = db.get(&feedback.id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let db = FeedbackDatabase::in_memory().unwrap();
        let id = FeedbackId::parse("fb-xyz123").unwrap();

        let deleted = db.delete(&id).expect("Should not error");
        assert!(!deleted, "Should return false when not found");
    }

    // =========================================================================
    // List Tests
    // =========================================================================

    #[test]
    fn test_list_all_feedback() {
        let db = FeedbackDatabase::in_memory().unwrap();

        // Save multiple feedbacks
        for i in 0..3 {
            let mut feedback = create_test_feedback();
            // Create unique ID for each
            feedback.id = FeedbackId::generate();
            feedback.user_description = format!("Feedback {}", i);
            db.save(&feedback).unwrap();
        }

        let all = db.list_all().expect("Should list all");
        assert_eq!(3, all.len());
    }

    #[test]
    fn test_list_by_status() {
        let db = FeedbackDatabase::in_memory().unwrap();

        // Save feedbacks with different statuses
        let mut submitted = create_test_feedback();
        submitted.status = FeedbackStatus::Submitted;
        db.save(&submitted).unwrap();

        let mut resolved = create_test_feedback();
        resolved.id = FeedbackId::generate();
        resolved.status = FeedbackStatus::Resolved;
        db.save(&resolved).unwrap();

        // Filter by status
        let submitted_list = db
            .list_by_status(FeedbackStatus::Submitted)
            .expect("Should list");
        assert_eq!(1, submitted_list.len());
        assert_eq!(FeedbackStatus::Submitted, submitted_list[0].status);

        let resolved_list = db
            .list_by_status(FeedbackStatus::Resolved)
            .expect("Should list");
        assert_eq!(1, resolved_list.len());
        assert_eq!(FeedbackStatus::Resolved, resolved_list[0].status);
    }

    #[test]
    fn test_count_by_status() {
        let db = FeedbackDatabase::in_memory().unwrap();

        // Save feedbacks with different statuses
        for _ in 0..3 {
            let mut fb = create_test_feedback();
            fb.id = FeedbackId::generate();
            fb.status = FeedbackStatus::Submitted;
            db.save(&fb).unwrap();
        }

        for _ in 0..2 {
            let mut fb = create_test_feedback();
            fb.id = FeedbackId::generate();
            fb.status = FeedbackStatus::Resolved;
            db.save(&fb).unwrap();
        }

        let counts = db.count_by_status().expect("Should count");
        assert_eq!(Some(&3), counts.get("submitted"));
        assert_eq!(Some(&2), counts.get("resolved"));
    }

    #[test]
    fn test_get_recent() {
        let db = FeedbackDatabase::in_memory().unwrap();

        // Save multiple feedbacks
        for i in 0..10 {
            let mut feedback = create_test_feedback();
            feedback.id = FeedbackId::generate();
            feedback.user_description = format!("Feedback {}", i);
            db.save(&feedback).unwrap();
        }

        // Get recent 5
        let recent = db.get_recent(5).expect("Should get recent");
        assert_eq!(5, recent.len());
    }

    // =========================================================================
    // Persistence Tests
    // =========================================================================

    #[test]
    fn test_persistence_across_connections() {
        let dir = tempdir().unwrap();

        // Create and save in first connection
        {
            let db = FeedbackDatabase::new(dir.path()).unwrap();
            let feedback = create_test_feedback();
            db.save(&feedback).unwrap();
        }

        // Reopen and verify
        {
            let db = FeedbackDatabase::new(dir.path()).unwrap();
            let all = db.list_all().unwrap();
            assert_eq!(1, all.len());
        }
    }

    // =========================================================================
    // Helper Functions
    // =========================================================================

    fn create_test_feedback() -> Feedback {
        Feedback {
            id: FeedbackId::parse("fb-abc123").unwrap(),
            timestamp: Utc::now(),
            user_description: "Test feedback description".to_string(),
            reproduction_steps: Some("1. Do this\n2. Do that".to_string()),
            context: FeedbackContext {
                timestamp: Utc::now(),
                cmdai_version: "1.0.0".to_string(),
                environment: EnvironmentInfo {
                    os: "macos".to_string(),
                    os_version: "14.0".to_string(),
                    arch: "arm64".to_string(),
                    shell: "zsh".to_string(),
                    terminal: "Terminal.app".to_string(),
                    rust_version: Some("1.75.0".to_string()),
                },
                command_info: CommandInfo {
                    user_prompt: "list files".to_string(),
                    generated_command: "ls -la".to_string(),
                    backend: "static".to_string(),
                    model: None,
                    command_history: vec![],
                },
                error_info: None,
                system_state: SystemState {
                    available_backends: vec!["static".to_string()],
                    cache_dir: PathBuf::from("/tmp/cache"),
                    config_file: None,
                    is_ci: false,
                    is_interactive: true,
                },
                git_context: None,
            },
            github_issue_url: None,
            status: FeedbackStatus::Submitted,
        }
    }
}
