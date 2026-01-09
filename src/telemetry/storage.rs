//! SQLite-based telemetry event storage

use super::Event;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use tokio::sync::Mutex;

/// SQLite-based storage for telemetry events
///
/// Events are stored locally in a SQLite database before being uploaded.
/// This provides:
/// - Offline operation support
/// - Reliable event queuing
/// - Air-gapped mode support
/// - Event export capability
///
/// The storage is thread-safe and async-ready via Mutex.
pub struct TelemetryStorage {
    conn: Mutex<Connection>,
}

impl TelemetryStorage {
    /// Create or open telemetry storage database
    ///
    /// Creates the database schema if it doesn't exist.
    ///
    /// # Schema
    ///
    /// ```sql
    /// CREATE TABLE events (
    ///     id TEXT PRIMARY KEY,
    ///     session_id TEXT NOT NULL,
    ///     timestamp TEXT NOT NULL,
    ///     event_type TEXT NOT NULL,
    ///     data TEXT NOT NULL
    /// );
    /// CREATE INDEX idx_timestamp ON events(timestamp);
    /// ```
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create telemetry directory")?;
        }

        let conn = Connection::open(&db_path).context("Failed to open telemetry database")?;

        // Create schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                data TEXT NOT NULL
            )",
            [],
        )
        .context("Failed to create events table")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON events(timestamp)",
            [],
        )
        .context("Failed to create timestamp index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_id ON events(session_id)",
            [],
        )
        .context("Failed to create session_id index")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Store a telemetry event
    pub async fn store_event(&self, event: &Event) -> Result<()> {
        let conn = self.conn.lock().await;

        let data = serde_json::to_string(event).context("Failed to serialize event")?;

        // Extract event type name for indexing
        let event_type = match &event.event_type {
            super::EventType::SessionStart { .. } => "session_start",
            super::EventType::SessionEnd { .. } => "session_end",
            super::EventType::CommandGeneration { .. } => "command_generation",
            super::EventType::SafetyValidation { .. } => "safety_validation",
            super::EventType::BackendError { .. } => "backend_error",
        };

        conn.execute(
            "INSERT INTO events (id, session_id, timestamp, event_type, data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.id.to_string(),
                event.session_id.as_str(),
                event.timestamp.to_rfc3339(),
                event_type,
                data,
            ],
        )
        .context("Failed to insert event")?;

        Ok(())
    }

    /// Get pending events for upload
    ///
    /// Returns events ordered by timestamp (oldest first), limited to `limit`.
    pub async fn get_pending_events(&self, limit: usize) -> Result<Vec<Event>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT data FROM events
                 ORDER BY timestamp ASC
                 LIMIT ?1",
            )
            .context("Failed to prepare query")?;

        let events = stmt
            .query_map([limit], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .filter_map(|result| {
                result
                    .ok()
                    .and_then(|data| serde_json::from_str(&data).ok())
            })
            .collect();

        Ok(events)
    }

    /// Get all events for a specific session
    pub async fn get_session_events(&self, session_id: &str) -> Result<Vec<Event>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                "SELECT data FROM events
                 WHERE session_id = ?1
                 ORDER BY timestamp ASC",
            )
            .context("Failed to prepare query")?;

        let events = stmt
            .query_map([session_id], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .filter_map(|result| {
                result
                    .ok()
                    .and_then(|data| serde_json::from_str(&data).ok())
            })
            .collect();

        Ok(events)
    }

    /// Delete successfully uploaded events
    pub async fn delete_events(&self, event_ids: &[String]) -> Result<()> {
        if event_ids.is_empty() {
            return Ok(());
        }

        let conn = self.conn.lock().await;

        let placeholders = event_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("DELETE FROM events WHERE id IN ({})", placeholders);

        let mut stmt = conn.prepare(&query).context("Failed to prepare delete")?;

        let params: Vec<&dyn rusqlite::ToSql> = event_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        stmt.execute(params.as_slice())
            .context("Failed to delete events")?;

        Ok(())
    }

    /// Get total event count
    pub async fn count_events(&self) -> Result<usize> {
        let conn = self.conn.lock().await;

        let count: usize = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .context("Failed to count events")?;

        Ok(count)
    }

    /// Clear all events (for testing or user-requested deletion)
    pub async fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute("DELETE FROM events", [])
            .context("Failed to clear events")?;

        Ok(())
    }

    /// Export all events as JSON
    pub async fn export_json(&self) -> Result<String> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT data FROM events ORDER BY timestamp ASC")
            .context("Failed to prepare export query")?;

        let events: Vec<Event> = stmt
            .query_map([], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .filter_map(|result| {
                result
                    .ok()
                    .and_then(|data| serde_json::from_str(&data).ok())
            })
            .collect();

        serde_json::to_string_pretty(&events).context("Failed to serialize events")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{EventType, SessionId};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let storage = TelemetryStorage::new(db_path.clone()).unwrap();

        // Verify database file was created
        assert!(db_path.exists());

        // Verify tables exist
        let count = storage.count_events().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_event() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = TelemetryStorage::new(db_path).unwrap();

        let session_id = SessionId::generate();
        let event = Event::new(
            session_id.clone(),
            EventType::SessionStart {
                version: "1.0.0".to_string(),
                platform: "linux".to_string(),
                shell_type: "bash".to_string(),
                backend_available: vec!["embedded".to_string()],
            },
        );

        // Store event
        storage.store_event(&event).await.unwrap();

        // Retrieve events
        let events = storage.get_pending_events(10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[tokio::test]
    async fn test_delete_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = TelemetryStorage::new(db_path).unwrap();

        let session_id = SessionId::generate();

        // Store multiple events
        for _ in 0..3 {
            let event = Event::new(
                session_id.clone(),
                EventType::CommandGeneration {
                    backend: "embedded".to_string(),
                    duration_ms: 100,
                    success: true,
                    error_category: None,
                },
            );
            storage.store_event(&event).await.unwrap();
        }

        let events = storage.get_pending_events(10).await.unwrap();
        assert_eq!(events.len(), 3);

        // Delete first two events
        let ids_to_delete: Vec<String> = events.iter().take(2).map(|e| e.id.to_string()).collect();
        storage.delete_events(&ids_to_delete).await.unwrap();

        // Verify only one event remains
        let remaining = storage.get_pending_events(10).await.unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[tokio::test]
    async fn test_export_json() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = TelemetryStorage::new(db_path).unwrap();

        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::SessionStart {
                version: "1.0.0".to_string(),
                platform: "linux".to_string(),
                shell_type: "bash".to_string(),
                backend_available: vec![],
            },
        );

        storage.store_event(&event).await.unwrap();

        let json = storage.export_json().await.unwrap();
        assert!(json.contains("session_start"));
        assert!(json.contains("1.0.0"));
    }
}
