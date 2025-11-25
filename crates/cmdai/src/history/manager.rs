//! Command history manager for persistent storage and retrieval
//!
//! Provides production-ready history management with:
//! - SQLite persistence with connection pooling
//! - FTS5 full-text search capabilities
//! - Semantic search with embedding support
//! - Performance monitoring and constitutional compliance
//! - Retention policy enforcement

use super::models::{CommandHistoryEntry, ExecutionMetadata};
use super::search::{SearchQuery, SearchResult};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Pagination result for history queries
#[derive(Debug, Clone)]
pub struct PaginatedHistory {
    pub entries: Vec<CommandHistoryEntry>,
    pub total_count: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Retention policy for automatic cleanup
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub max_entries: Option<usize>,
    pub max_age_days: Option<u32>,
    pub preserve_favorites: bool,
    pub preserve_frequently_used: bool,
}

/// Statistics about command history retention and usage
#[derive(Debug, Clone)]
pub struct RetentionStats {
    pub total_entries: usize,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
    pub size_bytes: u64,
    pub entries_last_24h: usize,
    pub entries_last_week: usize,
    pub entries_last_month: usize,
    pub top_commands: Vec<(String, usize)>,
}

/// History manager for command storage and retrieval with production features
pub struct HistoryManager {
    pool: Arc<Pool<SqliteConnectionManager>>,
    has_fts5: bool,
    has_embeddings: bool,
}

impl HistoryManager {
    /// Create a new history manager with connection pooling
    pub async fn new(database_path: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(database_path);

        // Configure connection pool for production
        let pool = Pool::builder()
            .max_size(4)  // Multiple connections for concurrent ops
            .min_idle(Some(2))
            .connection_timeout(Duration::from_secs(10))
            .build(manager)
            .context("Failed to create connection pool")?;

        // Initialize database schema
        let conn = pool.get()?;
        Self::initialize_schema(&conn)?;

        // Check capabilities
        let has_fts5 = Self::check_fts5_support(&conn)?;
        let has_embeddings = Self::check_embedding_support(&conn)?;

        info!(
            "History manager initialized with FTS5: {}, Embeddings: {}",
            has_fts5, has_embeddings
        );

        Ok(Self {
            pool: Arc::new(pool),
            has_fts5,
            has_embeddings,
        })
    }

    /// Initialize database schema with all required tables
    fn initialize_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            -- Main history table
            CREATE TABLE IF NOT EXISTS command_history (
                id TEXT PRIMARY KEY,
                command TEXT NOT NULL,
                user_input TEXT,
                explanation TEXT NOT NULL,
                shell_type TEXT NOT NULL DEFAULT 'bash',
                working_directory TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                execution_metadata TEXT,
                safety_metadata TEXT,
                tags TEXT,
                embedding_vector BLOB,
                relevance_score REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- FTS5 virtual table for full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS command_history_fts
            USING fts5(
                command, user_input, explanation, tags,
                content='command_history',
                content_rowid='rowid'
            );

            -- Triggers to keep FTS in sync
            CREATE TRIGGER IF NOT EXISTS command_history_ai
            AFTER INSERT ON command_history
            BEGIN
                INSERT INTO command_history_fts(rowid, command, user_input, explanation, tags)
                VALUES (new.rowid, new.command, new.user_input, new.explanation, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS command_history_au
            AFTER UPDATE ON command_history
            BEGIN
                UPDATE command_history_fts
                SET command = new.command,
                    user_input = new.user_input,
                    explanation = new.explanation,
                    tags = new.tags
                WHERE rowid = new.rowid;
            END;

            CREATE TRIGGER IF NOT EXISTS command_history_ad
            AFTER DELETE ON command_history
            BEGIN
                DELETE FROM command_history_fts WHERE rowid = old.rowid;
            END;

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON command_history(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_history_working_dir ON command_history(working_directory);
            CREATE INDEX IF NOT EXISTS idx_history_shell_type ON command_history(shell_type);
            "#
        )?;

        Ok(())
    }

    /// Check if FTS5 is supported
    fn check_fts5_support(conn: &Connection) -> Result<bool> {
        let result: Option<String> = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='command_history_fts'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result.is_some())
    }

    /// Check if embedding support is available
    fn check_embedding_support(conn: &Connection) -> Result<bool> {
        // Check if embedding_vector column exists
        let result: Option<i32> = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('command_history') WHERE name='embedding_vector'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result.map_or(false, |count| count > 0))
    }

    /// Store a command entry with timing validation
    pub async fn store_entry(&self, entry: &CommandHistoryEntry) -> Result<()> {
        let start = std::time::Instant::now();
        let pool = self.pool.clone();
        let entry = entry.clone();

        // Use tokio blocking for database operations
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            conn.execute(
                r#"INSERT INTO command_history
                   (id, command, user_input, explanation, shell_type, working_directory,
                    timestamp, execution_metadata, safety_metadata, tags, embedding_vector)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
                params![
                    &entry.id,
                    &entry.command,
                    &entry.user_input,
                    &entry.explanation,
                    &entry.shell_type.to_string(),
                    &entry.working_directory,
                    &entry.timestamp.to_rfc3339(),
                    entry
                        .execution_metadata
                        .as_ref()
                        .map(|m| serde_json::to_string(m).unwrap()),
                    entry
                        .safety_metadata
                        .as_ref()
                        .map(|m| serde_json::to_string(m).unwrap()),
                    Some(serde_json::to_string(&entry.tags).unwrap()),
                    entry
                        .embedding_vector
                        .as_ref()
                        .map(|v| { v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>() }),
                ],
            )?;

            Ok::<(), anyhow::Error>(())
        })
        .await??;

        let duration = start.elapsed();

        // Log performance for constitutional compliance
        if duration.as_millis() >= 10 {
            warn!(
                "History write took {}ms, exceeds constitutional requirement of 10ms",
                duration.as_millis()
            );
        } else {
            debug!("History write completed in {}ms", duration.as_millis());
        }

        Ok(())
    }

    /// Retrieve a specific entry by ID
    pub async fn get_entry(&self, id: &str) -> Result<Option<CommandHistoryEntry>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            let result = conn
                .query_row(
                    r#"SELECT id, command, user_input, explanation, shell_type,
                              working_directory, timestamp, execution_metadata,
                              safety_metadata, tags, embedding_vector
                       FROM command_history WHERE id = ?1"#,
                    params![id],
                    |row| {
                        Ok(CommandHistoryEntry {
                            id: row.get(0)?,
                            command: row.get(1)?,
                            user_input: row.get(2)?,
                            explanation: row.get(3)?,
                            shell_type: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                            working_directory: row.get(5)?,
                            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                                .unwrap()
                                .with_timezone(&Utc),
                            execution_metadata: row
                                .get::<_, Option<String>>(7)?
                                .and_then(|s| serde_json::from_str(&s).ok()),
                            safety_metadata: row
                                .get::<_, Option<String>>(8)?
                                .and_then(|s| serde_json::from_str(&s).ok()),
                            tags: row
                                .get::<_, Option<String>>(9)?
                                .and_then(|s| serde_json::from_str(&s).ok())
                                .unwrap_or_default(),
                            session_id: None,
                            hostname: None,
                            username: None,
                            embedding_vector: row.get::<_, Option<Vec<u8>>>(10)?.map(|bytes| {
                                bytes
                                    .chunks(4)
                                    .map(|chunk| {
                                        f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                                    })
                                    .collect()
                            }),
                            relevance_score: None,
                        })
                    },
                )
                .optional()?;

            Ok(result)
        })
        .await?
    }

    /// Search with FTS5 full-text search
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let start = std::time::Instant::now();
        let pool = self.pool.clone();
        let query = query.clone();

        let results = tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            let sql = String::from(
                r#"SELECT h.id, h.command, h.user_input, h.explanation, h.shell_type,
                          h.working_directory, h.timestamp, h.execution_metadata,
                          h.safety_metadata, h.tags, h.embedding_vector,
                          rank
                   FROM command_history h
                   INNER JOIN command_history_fts fts ON h.rowid = fts.rowid
                   WHERE command_history_fts MATCH ?1
                   ORDER BY rank
                   LIMIT 100"#,
            );

            let mut stmt = conn.prepare(&sql)?;
            let results = stmt.query_map(params![query.text], |row| {
                let entry = CommandHistoryEntry {
                    id: row.get(0)?,
                    command: row.get(1)?,
                    user_input: row.get(2)?,
                    explanation: row.get(3)?,
                    shell_type: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                    working_directory: row.get(5)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    execution_metadata: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    safety_metadata: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    tags: row
                        .get::<_, Option<String>>(9)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    session_id: None,
                    hostname: None,
                    username: None,
                    embedding_vector: None,
                    relevance_score: None,
                };

                let rank: f64 = row.get(11)?;

                Ok(SearchResult {
                    entry,
                    similarity_score: rank.abs(), // Convert rank to positive score
                    matching_context: String::new(),
                    explanation: String::new(),
                })
            })?;

            let mut search_results = Vec::new();
            for result in results {
                search_results.push(result?);
            }

            Ok::<Vec<SearchResult>, anyhow::Error>(search_results)
        })
        .await??;

        let duration = start.elapsed();

        // Log performance for constitutional compliance
        if duration.as_millis() >= 50 {
            warn!(
                "History search took {}ms, exceeds constitutional requirement of 50ms",
                duration.as_millis()
            );
        }

        Ok(results)
    }

    /// Semantic search using embeddings
    pub async fn semantic_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        if !self.has_embeddings {
            return Ok(Vec::new());
        }

        // Placeholder for semantic search implementation
        // Will be fully implemented when embedding model is integrated
        self.search(query).await
    }

    /// Get paginated history
    pub async fn get_history_paginated(
        &self,
        page: usize,
        page_size: usize,
    ) -> Result<PaginatedHistory> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            // Get total count
            let total_count: usize =
                conn.query_row("SELECT COUNT(*) FROM command_history", [], |row| row.get(0))?;

            // Get page of entries
            let offset = page * page_size;
            let mut stmt = conn.prepare(
                r#"SELECT id, command, user_input, explanation, shell_type,
                          working_directory, timestamp, execution_metadata,
                          safety_metadata, tags
                   FROM command_history
                   ORDER BY timestamp DESC
                   LIMIT ?1 OFFSET ?2"#,
            )?;

            let entries = stmt
                .query_map(params![page_size, offset], |row| {
                    Ok(CommandHistoryEntry {
                        id: row.get(0)?,
                        command: row.get(1)?,
                        user_input: row.get(2)?,
                        explanation: row.get(3)?,
                        shell_type: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                        working_directory: row.get(5)?,
                        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                            .unwrap()
                            .with_timezone(&Utc),
                        execution_metadata: row
                            .get::<_, Option<String>>(7)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        safety_metadata: row
                            .get::<_, Option<String>>(8)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        tags: row
                            .get::<_, Option<String>>(9)?
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .unwrap_or_default(),
                        session_id: None,
                        hostname: None,
                        username: None,
                        embedding_vector: None,
                        relevance_score: None,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(PaginatedHistory {
                entries,
                total_count,
                page,
                page_size,
            })
        })
        .await?
    }

    /// Delete a specific entry
    pub async fn delete_entry(&self, id: &str) -> Result<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            conn.execute("DELETE FROM command_history WHERE id = ?1", params![id])?;
            Ok(())
        })
        .await?
    }

    /// Update execution metadata for an entry
    pub async fn update_execution_metadata(
        &self,
        id: &str,
        metadata: ExecutionMetadata,
    ) -> Result<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let metadata_json = serde_json::to_string(&metadata)?;

            conn.execute(
                "UPDATE command_history SET execution_metadata = ?1 WHERE id = ?2",
                params![metadata_json, id],
            )?;

            Ok(())
        })
        .await?
    }

    /// Apply retention policy to clean up old entries
    pub async fn apply_retention_policy(&self, policy: &RetentionPolicy) -> Result<usize> {
        let pool = self.pool.clone();
        let policy = policy.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut removed_count = 0;

            info!(
                max_entries = ?policy.max_entries,
                max_age_days = ?policy.max_age_days,
                preserve_favorites = policy.preserve_favorites,
                preserve_frequently_used = policy.preserve_frequently_used,
                "Applying retention policy"
            );

            // Clean up by age if specified
            if let Some(max_age_days) = policy.max_age_days {
                let cutoff_date = Utc::now() - chrono::Duration::days(max_age_days as i64);
                let cutoff_str = cutoff_date.to_rfc3339();

                let mut delete_sql = r#"
                    DELETE FROM command_history
                    WHERE timestamp < ?1
                "#
                .to_string();

                // Add preservation conditions
                if policy.preserve_favorites {
                    delete_sql.push_str(
                        " AND id NOT IN (SELECT entry_id FROM favorites WHERE deleted = 0)",
                    );
                }

                if policy.preserve_frequently_used {
                    delete_sql.push_str(
                        " AND id NOT IN (
                        SELECT id FROM command_history
                        WHERE command IN (
                            SELECT command FROM command_history
                            GROUP BY command
                            HAVING COUNT(*) >= 5
                        )
                    )",
                    );
                }

                let age_removed = conn.execute(&delete_sql, params![cutoff_str])?;
                removed_count += age_removed;

                debug!(
                    removed_by_age = age_removed,
                    cutoff_date = %cutoff_date,
                    "Cleaned up entries by age"
                );
            }

            // Clean up by count if specified
            if let Some(max_entries) = policy.max_entries {
                let current_count: usize =
                    conn.query_row("SELECT COUNT(*) FROM command_history", [], |row| row.get(0))?;

                if current_count > max_entries {
                    let excess = current_count - max_entries;

                    let mut delete_sql = r#"
                        DELETE FROM command_history
                        WHERE id IN (
                            SELECT id FROM command_history
                            ORDER BY timestamp ASC
                            LIMIT ?1
                        )
                    "#
                    .to_string();

                    // Exclude preserved entries
                    if policy.preserve_favorites || policy.preserve_frequently_used {
                        delete_sql = r#"
                            DELETE FROM command_history
                            WHERE id IN (
                                SELECT id FROM command_history
                                WHERE 1=1
                        "#
                        .to_string();

                        if policy.preserve_favorites {
                            delete_sql.push_str(
                                " AND id NOT IN (SELECT entry_id FROM favorites WHERE deleted = 0)",
                            );
                        }

                        if policy.preserve_frequently_used {
                            delete_sql.push_str(
                                " AND id NOT IN (
                                SELECT id FROM command_history
                                WHERE command IN (
                                    SELECT command FROM command_history
                                    GROUP BY command
                                    HAVING COUNT(*) >= 5
                                )
                            )",
                            );
                        }

                        delete_sql.push_str(
                            "
                                ORDER BY timestamp ASC
                                LIMIT ?1
                            )
                        ",
                        );
                    }

                    let count_removed = conn.execute(&delete_sql, params![excess])?;
                    removed_count += count_removed;

                    debug!(
                        removed_by_count = count_removed,
                        target_count = max_entries,
                        "Cleaned up entries by count"
                    );
                }
            }

            // Update FTS5 index if it exists
            if Self::check_fts5_support(&conn).unwrap_or(false) {
                if let Err(e) = conn.execute(
                    "INSERT INTO command_history_fts(command_history_fts) VALUES('rebuild')",
                    [],
                ) {
                    warn!("Failed to rebuild FTS5 index after cleanup: {}", e);
                }
            }

            info!(total_removed = removed_count, "Retention policy applied");
            Ok(removed_count)
        })
        .await?
    }

    /// Clean up entries older than specified days
    pub async fn cleanup_old_entries(&self, days: u32) -> Result<usize> {
        let policy = RetentionPolicy {
            max_age_days: Some(days),
            max_entries: None,
            preserve_favorites: true, // Default to preserving favorites
            preserve_frequently_used: true, // Default to preserving frequent commands
        };

        self.apply_retention_policy(&policy).await
    }

    /// Clean up entries to maintain maximum count
    pub async fn cleanup_excess_entries(&self, max_entries: usize) -> Result<usize> {
        let policy = RetentionPolicy {
            max_age_days: None,
            max_entries: Some(max_entries),
            preserve_favorites: true,
            preserve_frequently_used: true,
        };

        self.apply_retention_policy(&policy).await
    }

    /// Clean up all entries (careful - this removes everything!)
    pub async fn cleanup_all_entries(&self) -> Result<usize> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            warn!("Cleaning up ALL command history entries");

            let removed_count = conn.execute("DELETE FROM command_history", [])?;

            // Clear FTS5 index if it exists
            if Self::check_fts5_support(&conn).unwrap_or(false) {
                if let Err(e) = conn.execute("DELETE FROM command_history_fts", []) {
                    warn!("Failed to clear FTS5 index: {}", e);
                }
            }

            info!(removed_count = removed_count, "All history entries cleared");
            Ok(removed_count)
        })
        .await?
    }

    /// Get retention statistics
    pub async fn get_retention_stats(&self) -> Result<RetentionStats> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            let total_entries: usize = conn.query_row(
                "SELECT COUNT(*) FROM command_history",
                [],
                |row| row.get(0),
            )?;

            let oldest_entry: Option<DateTime<Utc>> = conn.query_row(
                "SELECT MIN(timestamp) FROM command_history",
                [],
                |row| {
                    let timestamp: Option<String> = row.get(0)?;
                    Ok(timestamp.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))))
                },
            ).optional()?.flatten();

            let newest_entry: Option<DateTime<Utc>> = conn.query_row(
                "SELECT MAX(timestamp) FROM command_history",
                [],
                |row| {
                    let timestamp: Option<String> = row.get(0)?;
                    Ok(timestamp.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))))
                },
            ).optional()?.flatten();

            let size_bytes: i64 = conn.query_row(
                "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            // Count by age ranges
            let now = Utc::now();
            let last_24h = (now - chrono::Duration::hours(24)).to_rfc3339();
            let last_week = (now - chrono::Duration::weeks(1)).to_rfc3339();
            let last_month = (now - chrono::Duration::weeks(4)).to_rfc3339();

            let entries_last_24h: usize = conn.query_row(
                "SELECT COUNT(*) FROM command_history WHERE timestamp > ?1",
                params![last_24h],
                |row| row.get(0),
            )?;

            let entries_last_week: usize = conn.query_row(
                "SELECT COUNT(*) FROM command_history WHERE timestamp > ?1",
                params![last_week],
                |row| row.get(0),
            )?;

            let entries_last_month: usize = conn.query_row(
                "SELECT COUNT(*) FROM command_history WHERE timestamp > ?1",
                params![last_month],
                |row| row.get(0),
            )?;

            // Get top commands
            let mut stmt = conn.prepare(
                "SELECT command, COUNT(*) as freq FROM command_history GROUP BY command ORDER BY freq DESC LIMIT 10"
            )?;

            let top_commands: Vec<(String, usize)> = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })?.collect::<Result<Vec<_>, _>>()?;

            Ok(RetentionStats {
                total_entries,
                oldest_entry,
                newest_entry,
                size_bytes: size_bytes as u64,
                entries_last_24h,
                entries_last_week,
                entries_last_month,
                top_commands,
            })
        })
        .await?
    }

    /// Vacuum the database to reclaim space after cleanup
    pub async fn vacuum_database(&self) -> Result<u64> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            // Get size before vacuum
            let size_before: i64 = conn
                .query_row(
                    "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            info!(
                "Starting database vacuum (size before: {} bytes)",
                size_before
            );

            // Perform vacuum
            conn.execute("VACUUM", [])?;

            // Get size after vacuum
            let size_after: i64 = conn
                .query_row(
                    "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let space_saved = (size_before - size_after).max(0) as u64;

            info!(
                size_before = size_before,
                size_after = size_after,
                space_saved = space_saved,
                "Database vacuum completed"
            );

            Ok(space_saved)
        })
        .await?
    }

    /// Check if FTS5 is supported
    pub async fn has_fts_support(&self) -> Result<bool> {
        Ok(self.has_fts5)
    }

    /// Check if embeddings are supported
    pub async fn has_embedding_support(&self) -> Result<bool> {
        Ok(self.has_embeddings)
    }

    /// Get connection pool size
    pub fn connection_pool_size(&self) -> u32 {
        self.pool.state().connections as u32
    }
}

// Backward compatibility support
impl HistoryManager {
    /// Create a new history manager (sync version for compatibility)
    pub fn new_sync(database_path: &str) -> Result<Self> {
        // For backward compatibility, use blocking runtime
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(Self::new(database_path))
    }

    /// Get recent history (convenience method)
    pub async fn get_history(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<CommandHistoryEntry>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            let mut stmt = conn.prepare(
                r#"SELECT id, command, user_input, explanation, shell_type,
                          working_directory, timestamp, execution_metadata,
                          safety_metadata, tags
                   FROM command_history
                   ORDER BY timestamp DESC
                   LIMIT ?1 OFFSET ?2"#,
            )?;

            let entries = stmt
                .query_map(params![limit, offset], |row| {
                    Ok(CommandHistoryEntry {
                        id: row.get(0)?,
                        command: row.get(1)?,
                        user_input: row.get(2)?,
                        explanation: row.get(3)?,
                        shell_type: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                        working_directory: row.get(5)?,
                        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                            .unwrap()
                            .with_timezone(&Utc),
                        execution_metadata: row
                            .get::<_, Option<String>>(7)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        safety_metadata: row
                            .get::<_, Option<String>>(8)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        tags: row
                            .get::<_, Option<String>>(9)?
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .unwrap_or_default(),
                        session_id: None,
                        hostname: None,
                        username: None,
                        embedding_vector: None,
                        relevance_score: None,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(entries)
        })
        .await?
    }
}
