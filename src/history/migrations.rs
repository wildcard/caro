//! Database Migration System
//!
//! Provides versioned database schema management with forward and backward
//! migration support for the command history SQLite database.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Database schema version and migration management
#[derive(Debug)]
pub struct MigrationManager {
    migrations: Vec<Migration>,
}

/// Individual database migration
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub forward_sql: Vec<String>,
    pub backward_sql: Vec<String>,
    pub required_features: Vec<DatabaseFeature>,
}

/// Database features that can be checked for availability
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseFeature {
    FTS5,
    JSONExtract,
    WindowFunctions,
    CTERecursive,
    GeneratedColumns,
}

/// Migration execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub version: u32,
    pub name: String,
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Current database state
#[derive(Debug, Clone)]
pub struct DatabaseState {
    pub current_version: u32,
    pub available_features: HashMap<DatabaseFeature, bool>,
    pub migration_history: Vec<MigrationResult>,
    pub needs_migration: bool,
}

impl MigrationManager {
    /// Create a new migration manager with all defined migrations
    pub fn new() -> Self {
        let mut migrations = Vec::new();

        // Migration 1: Initial schema
        migrations.push(Migration {
            version: 1,
            name: "initial_schema".to_string(),
            description: "Create initial command history tables".to_string(),
            forward_sql: vec![
                r#"
                CREATE TABLE IF NOT EXISTS command_history (
                    id TEXT PRIMARY KEY,
                    command TEXT NOT NULL,
                    user_input TEXT,
                    explanation TEXT NOT NULL,
                    shell_type TEXT NOT NULL,
                    working_directory TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    execution_metadata TEXT,
                    safety_metadata TEXT,
                    tags TEXT
                );
                "#.to_string(),
                r#"
                CREATE INDEX IF NOT EXISTS idx_command_history_timestamp 
                ON command_history(timestamp);
                "#.to_string(),
                r#"
                CREATE INDEX IF NOT EXISTS idx_command_history_shell_type 
                ON command_history(shell_type);
                "#.to_string(),
            ],
            backward_sql: vec![
                "DROP INDEX IF EXISTS idx_command_history_shell_type;".to_string(),
                "DROP INDEX IF EXISTS idx_command_history_timestamp;".to_string(),
                "DROP TABLE IF EXISTS command_history;".to_string(),
            ],
            required_features: vec![],
        });

        // Migration 2: Add FTS5 full-text search
        migrations.push(Migration {
            version: 2,
            name: "add_fts5_search".to_string(),
            description: "Add FTS5 virtual table for full-text search".to_string(),
            forward_sql: vec![
                r#"
                CREATE VIRTUAL TABLE IF NOT EXISTS command_history_fts USING fts5(
                    command, user_input, explanation, tags,
                    content='command_history',
                    content_rowid='rowid'
                );
                "#.to_string(),
                r#"
                INSERT INTO command_history_fts(command_history_fts) VALUES('rebuild');
                "#.to_string(),
            ],
            backward_sql: vec![
                "DROP TABLE IF EXISTS command_history_fts;".to_string(),
            ],
            required_features: vec![DatabaseFeature::FTS5],
        });

        // Migration 3: Add embedding support
        migrations.push(Migration {
            version: 3,
            name: "add_embedding_support".to_string(),
            description: "Add embedding vector storage for semantic search".to_string(),
            forward_sql: vec![
                r#"
                ALTER TABLE command_history 
                ADD COLUMN embedding_vector BLOB;
                "#.to_string(),
                r#"
                ALTER TABLE command_history 
                ADD COLUMN relevance_score REAL;
                "#.to_string(),
                r#"
                CREATE TABLE IF NOT EXISTS embedding_cache (
                    content_hash TEXT PRIMARY KEY,
                    embedding_vector BLOB NOT NULL,
                    created_at TEXT NOT NULL,
                    access_count INTEGER DEFAULT 0,
                    last_accessed TEXT NOT NULL
                );
                "#.to_string(),
                r#"
                CREATE INDEX IF NOT EXISTS idx_embedding_cache_last_accessed 
                ON embedding_cache(last_accessed);
                "#.to_string(),
            ],
            backward_sql: vec![
                "DROP INDEX IF EXISTS idx_embedding_cache_last_accessed;".to_string(),
                "DROP TABLE IF EXISTS embedding_cache;".to_string(),
                // Note: SQLite doesn't support DROP COLUMN, so we skip removing columns
            ],
            required_features: vec![],
        });

        // Migration 4: Add performance monitoring tables
        migrations.push(Migration {
            version: 4,
            name: "add_performance_monitoring".to_string(),
            description: "Add backend performance metrics storage".to_string(),
            forward_sql: vec![
                r#"
                CREATE TABLE IF NOT EXISTS backend_metrics (
                    backend_name TEXT PRIMARY KEY,
                    avg_response_time_ms INTEGER,
                    success_rate REAL,
                    availability REAL,
                    last_health_check TEXT,
                    metrics_json TEXT
                );
                "#.to_string(),
                r#"
                CREATE TABLE IF NOT EXISTS configuration (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    last_updated TEXT NOT NULL
                );
                "#.to_string(),
            ],
            backward_sql: vec![
                "DROP TABLE IF EXISTS configuration;".to_string(),
                "DROP TABLE IF EXISTS backend_metrics;".to_string(),
            ],
            required_features: vec![],
        });

        // Migration 5: Add advanced indexing and optimization
        migrations.push(Migration {
            version: 5,
            name: "optimize_indexes".to_string(),
            description: "Add optimized indexes for common query patterns".to_string(),
            forward_sql: vec![
                r#"
                CREATE INDEX IF NOT EXISTS idx_command_history_working_directory 
                ON command_history(working_directory);
                "#.to_string(),
                r#"
                CREATE INDEX IF NOT EXISTS idx_command_history_compound 
                ON command_history(shell_type, timestamp);
                "#.to_string(),
                r#"
                CREATE INDEX IF NOT EXISTS idx_embedding_cache_access_pattern 
                ON embedding_cache(access_count, last_accessed);
                "#.to_string(),
            ],
            backward_sql: vec![
                "DROP INDEX IF EXISTS idx_embedding_cache_access_pattern;".to_string(),
                "DROP INDEX IF EXISTS idx_command_history_compound;".to_string(),
                "DROP INDEX IF EXISTS idx_command_history_working_directory;".to_string(),
            ],
            required_features: vec![],
        });

        Self { migrations }
    }

    /// Initialize migration tracking table
    pub fn initialize_migration_table(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                executed_at TEXT NOT NULL,
                execution_time_ms INTEGER NOT NULL,
                checksum TEXT NOT NULL
            );
            "#,
            [],
        ).context("Failed to create schema_migrations table")?;

        debug!("Migration tracking table initialized");
        Ok(())
    }

    /// Get current database version
    pub fn get_current_version(&self, conn: &Connection) -> Result<u32> {
        let version: Option<u32> = conn
            .query_row(
                "SELECT MAX(version) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .optional()
            .context("Failed to query current schema version")?
            .flatten();

        Ok(version.unwrap_or(0))
    }

    /// Check database feature availability
    pub fn check_features(&self, conn: &Connection) -> Result<HashMap<DatabaseFeature, bool>> {
        let mut features = HashMap::new();

        // Check FTS5 support
        let fts5_available = conn
            .prepare("SELECT fts5(?)")
            .and_then(|mut stmt| stmt.query_row(params!["test"], |_| Ok(())))
            .is_ok();
        features.insert(DatabaseFeature::FTS5, fts5_available);

        // Check JSON support
        let json_available = conn
            .prepare("SELECT json_extract('{}', '$')")
            .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
            .is_ok();
        features.insert(DatabaseFeature::JSONExtract, json_available);

        // Check window functions (SQLite 3.25.0+)
        let window_available = conn
            .prepare("SELECT row_number() OVER () FROM (SELECT 1) AS t")
            .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
            .is_ok();
        features.insert(DatabaseFeature::WindowFunctions, window_available);

        // Check recursive CTE support
        let cte_available = conn
            .prepare("WITH RECURSIVE test(x) AS (SELECT 1) SELECT * FROM test")
            .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
            .is_ok();
        features.insert(DatabaseFeature::CTERecursive, cte_available);

        // Check generated columns (SQLite 3.31.0+)
        let generated_available = conn
            .execute(
                "CREATE TEMP TABLE test_generated (id INTEGER, computed INTEGER GENERATED ALWAYS AS (id * 2))",
                [],
            )
            .is_ok();
        features.insert(DatabaseFeature::GeneratedColumns, generated_available);

        debug!("Database features checked: {:?}", features);
        Ok(features)
    }

    /// Get database state
    pub fn get_database_state(&self, conn: &Connection) -> Result<DatabaseState> {
        self.initialize_migration_table(conn)?;
        
        let current_version = self.get_current_version(conn)?;
        let available_features = self.check_features(conn)?;
        let latest_version = self.migrations.iter().map(|m| m.version).max().unwrap_or(0);
        
        let migration_history = self.get_migration_history(conn)?;

        Ok(DatabaseState {
            current_version,
            available_features,
            migration_history,
            needs_migration: current_version < latest_version,
        })
    }

    /// Apply all pending migrations
    pub fn migrate_to_latest(&self, conn: &Connection) -> Result<Vec<MigrationResult>> {
        self.initialize_migration_table(conn)?;
        
        let current_version = self.get_current_version(conn)?;
        let available_features = self.check_features(conn)?;
        let mut results = Vec::new();

        info!(
            current_version = current_version,
            latest_version = self.migrations.iter().map(|m| m.version).max().unwrap_or(0),
            "Starting database migration"
        );

        for migration in &self.migrations {
            if migration.version <= current_version {
                continue;
            }

            // Check if required features are available
            let missing_features: Vec<_> = migration.required_features
                .iter()
                .filter(|feature| !*available_features.get(feature).unwrap_or(&false))
                .collect();

            if !missing_features.is_empty() {
                warn!(
                    migration_version = migration.version,
                    missing_features = ?missing_features,
                    "Skipping migration due to missing database features"
                );
                continue;
            }

            let result = self.apply_migration(conn, migration)?;
            results.push(result);
        }

        if !results.is_empty() {
            info!(
                migrations_applied = results.len(),
                final_version = self.get_current_version(conn)?,
                "Database migration completed"
            );
        }

        Ok(results)
    }

    /// Apply a specific migration
    fn apply_migration(&self, conn: &Connection, migration: &Migration) -> Result<MigrationResult> {
        let start_time = std::time::Instant::now();
        let executed_at = Utc::now();

        info!(
            version = migration.version,
            name = migration.name,
            description = migration.description,
            "Applying migration"
        );

        // Begin transaction
        let tx = conn.unchecked_transaction()?;

        let mut success = true;
        let mut error_message = None;

        // Execute forward SQL statements
        for sql in &migration.forward_sql {
            if let Err(e) = tx.execute(sql, []) {
                success = false;
                error_message = Some(format!("SQL execution failed: {}", e));
                warn!(
                    migration_version = migration.version,
                    sql = sql,
                    error = %e,
                    "Migration SQL failed"
                );
                break;
            }
        }

        if success {
            // Record successful migration
            let checksum = self.calculate_migration_checksum(migration);
            tx.execute(
                r#"
                INSERT INTO schema_migrations 
                (version, name, description, executed_at, execution_time_ms, checksum)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
                params![
                    migration.version,
                    migration.name,
                    migration.description,
                    executed_at.to_rfc3339(),
                    start_time.elapsed().as_millis() as i64,
                    checksum
                ],
            ).context("Failed to record migration")?;

            tx.commit().context("Failed to commit migration transaction")?;
        } else {
            tx.rollback().context("Failed to rollback failed migration")?;
        }

        Ok(MigrationResult {
            version: migration.version,
            name: migration.name.clone(),
            executed_at,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            success,
            error_message,
        })
    }

    /// Rollback to a specific version
    pub fn rollback_to_version(&self, conn: &Connection, target_version: u32) -> Result<Vec<MigrationResult>> {
        let current_version = self.get_current_version(conn)?;
        
        if target_version >= current_version {
            return Ok(vec![]);
        }

        info!(
            current_version = current_version,
            target_version = target_version,
            "Starting database rollback"
        );

        let mut results = Vec::new();

        // Apply rollbacks in reverse order
        for migration in self.migrations.iter().rev() {
            if migration.version <= target_version || migration.version > current_version {
                continue;
            }

            let result = self.rollback_migration(conn, migration)?;
            results.push(result);
        }

        if !results.is_empty() {
            info!(
                rollbacks_applied = results.len(),
                final_version = self.get_current_version(conn)?,
                "Database rollback completed"
            );
        }

        Ok(results)
    }

    /// Rollback a specific migration
    fn rollback_migration(&self, conn: &Connection, migration: &Migration) -> Result<MigrationResult> {
        let start_time = std::time::Instant::now();
        let executed_at = Utc::now();

        info!(
            version = migration.version,
            name = migration.name,
            "Rolling back migration"
        );

        let tx = conn.unchecked_transaction()?;

        let mut success = true;
        let mut error_message = None;

        // Execute backward SQL statements
        for sql in &migration.backward_sql {
            if let Err(e) = tx.execute(sql, []) {
                success = false;
                error_message = Some(format!("Rollback SQL failed: {}", e));
                warn!(
                    migration_version = migration.version,
                    sql = sql,
                    error = %e,
                    "Migration rollback SQL failed"
                );
                break;
            }
        }

        if success {
            // Remove migration record
            tx.execute(
                "DELETE FROM schema_migrations WHERE version = ?",
                params![migration.version],
            ).context("Failed to remove migration record")?;

            tx.commit().context("Failed to commit rollback transaction")?;
        } else {
            tx.rollback().context("Failed to rollback failed rollback")?;
        }

        Ok(MigrationResult {
            version: migration.version,
            name: format!("rollback_{}", migration.name),
            executed_at,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            success,
            error_message,
        })
    }

    /// Get migration history
    pub fn get_migration_history(&self, conn: &Connection) -> Result<Vec<MigrationResult>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT version, name, description, executed_at, execution_time_ms
            FROM schema_migrations 
            ORDER BY version
            "#
        )?;

        let results = stmt.query_map([], |row| {
            Ok(MigrationResult {
                version: row.get(0)?,
                name: row.get(1)?,
                executed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(3, format!("Invalid timestamp: {}", e).into(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                execution_time_ms: row.get(4)?,
                success: true, // Only successful migrations are recorded
                error_message: None,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Validate database schema integrity
    pub fn validate_schema(&self, conn: &Connection) -> Result<bool> {
        let state = self.get_database_state(conn)?;
        
        // Check if all expected tables exist
        let expected_tables = vec![
            "command_history", 
            "schema_migrations",
            "embedding_cache",
            "backend_metrics",
            "configuration"
        ];

        for table in expected_tables {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?",
                params![table],
                |row| row.get(0),
            ).unwrap_or(false);

            if !exists && state.current_version >= self.get_table_introduction_version(table) {
                warn!(table = table, "Expected table is missing");
                return Ok(false);
            }
        }

        // Validate FTS5 table if it should exist
        if state.current_version >= 2 && *state.available_features.get(&DatabaseFeature::FTS5).unwrap_or(&false) {
            let fts_exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='command_history_fts'",
                [],
                |row| row.get(0),
            ).unwrap_or(false);

            if !fts_exists {
                warn!("FTS5 table missing despite version 2+ and FTS5 support");
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Calculate migration checksum for integrity verification
    fn calculate_migration_checksum(&self, migration: &Migration) -> String {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(migration.version.to_string());
        hasher.update(&migration.name);
        hasher.update(&migration.description);
        
        for sql in &migration.forward_sql {
            hasher.update(sql);
        }
        
        format!("{:x}", hasher.finalize())
    }

    /// Get the version when a table was introduced
    fn get_table_introduction_version(&self, table_name: &str) -> u32 {
        match table_name {
            "command_history" | "schema_migrations" => 1,
            "command_history_fts" => 2,
            "embedding_cache" => 3,
            "backend_metrics" | "configuration" => 4,
            _ => u32::MAX,
        }
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_manager_creation() {
        let manager = MigrationManager::new();
        assert!(!manager.migrations.is_empty());
        assert_eq!(manager.migrations[0].version, 1);
    }

    #[test]
    fn test_database_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        let manager = MigrationManager::new();
        
        manager.initialize_migration_table(&conn).unwrap();
        
        // Verify table was created
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| row.get(0),
        ).unwrap();
        
        assert!(table_exists);
    }

    #[test]
    fn test_feature_detection() {
        let conn = Connection::open_in_memory().unwrap();
        let manager = MigrationManager::new();
        
        let features = manager.check_features(&conn).unwrap();
        
        // These features should be available in modern SQLite
        assert!(features.contains_key(&DatabaseFeature::JSONExtract));
        assert!(features.contains_key(&DatabaseFeature::WindowFunctions));
    }

    #[test]
    fn test_migration_application() {
        let conn = Connection::open_in_memory().unwrap();
        let manager = MigrationManager::new();
        
        let results = manager.migrate_to_latest(&conn).unwrap();
        
        // Should have applied some migrations
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.success));
        
        // Verify final version
        let final_version = manager.get_current_version(&conn).unwrap();
        assert!(final_version > 0);
    }

    #[test]
    fn test_schema_validation() {
        let conn = Connection::open_in_memory().unwrap();
        let manager = MigrationManager::new();
        
        // Apply all migrations
        manager.migrate_to_latest(&conn).unwrap();
        
        // Schema should be valid
        assert!(manager.validate_schema(&conn).unwrap());
    }

    #[test]
    fn test_migration_rollback() {
        let conn = Connection::open_in_memory().unwrap();
        let manager = MigrationManager::new();
        
        // Apply migrations
        manager.migrate_to_latest(&conn).unwrap();
        let initial_version = manager.get_current_version(&conn).unwrap();
        
        // Rollback to version 1
        let rollback_results = manager.rollback_to_version(&conn, 1).unwrap();
        
        if initial_version > 1 {
            assert!(!rollback_results.is_empty());
            assert_eq!(manager.get_current_version(&conn).unwrap(), 1);
        }
    }

    #[test]
    fn test_migration_checksum() {
        let manager = MigrationManager::new();
        let migration = &manager.migrations[0];
        
        let checksum1 = manager.calculate_migration_checksum(migration);
        let checksum2 = manager.calculate_migration_checksum(migration);
        
        assert_eq!(checksum1, checksum2);
        assert!(!checksum1.is_empty());
    }
}