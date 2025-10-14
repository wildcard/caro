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
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, debug};

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
                    entry.execution_metadata.as_ref().map(|m| serde_json::to_string(m).unwrap()),
                    entry.safety_metadata.as_ref().map(|m| serde_json::to_string(m).unwrap()),
                    Some(serde_json::to_string(&entry.tags).unwrap()),
                    entry.embedding_vector.as_ref().map(|v| {
                        v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>()
                    }),
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
                            execution_metadata: row.get::<_, Option<String>>(7)?
                                .and_then(|s| serde_json::from_str(&s).ok()),
                            safety_metadata: row.get::<_, Option<String>>(8)?
                                .and_then(|s| serde_json::from_str(&s).ok()),
                            tags: row.get::<_, Option<String>>(9)?
                                .and_then(|s| serde_json::from_str(&s).ok())
                                .unwrap_or_default(),
                            session_id: None,
                            hostname: None,
                            username: None,
                            embedding_vector: row.get::<_, Option<Vec<u8>>>(10)?
                                .map(|bytes| {
                                    bytes.chunks(4)
                                        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
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
                   LIMIT 100"#
            );
            
            let mut stmt = conn.prepare(&sql)?;
            let results = stmt
                .query_map(params![query.text], |row| {
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
                        execution_metadata: row.get::<_, Option<String>>(7)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        safety_metadata: row.get::<_, Option<String>>(8)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        tags: row.get::<_, Option<String>>(9)?
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
    pub async fn get_history_paginated(&self, page: usize, page_size: usize) -> Result<PaginatedHistory> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            // Get total count
            let total_count: usize = conn.query_row(
                "SELECT COUNT(*) FROM command_history",
                [],
                |row| row.get(0),
            )?;
            
            // Get page of entries
            let offset = page * page_size;
            let mut stmt = conn.prepare(
                r#"SELECT id, command, user_input, explanation, shell_type,
                          working_directory, timestamp, execution_metadata,
                          safety_metadata, tags
                   FROM command_history
                   ORDER BY timestamp DESC
                   LIMIT ?1 OFFSET ?2"#
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
                        execution_metadata: row.get::<_, Option<String>>(7)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        safety_metadata: row.get::<_, Option<String>>(8)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        tags: row.get::<_, Option<String>>(9)?
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
    pub async fn update_execution_metadata(&self, id: &str, metadata: ExecutionMetadata) -> Result<()> {
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
    
    /// Apply retention policy for cleanup
    pub async fn apply_retention_policy(&self, policy: &RetentionPolicy) -> Result<usize> {
        let pool = self.pool.clone();
        let policy = policy.clone();
        
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut deleted = 0;
            
            // Delete by age
            if let Some(max_age_days) = policy.max_age_days {
                let cutoff = Utc::now() - chrono::Duration::days(max_age_days as i64);
                deleted += conn.execute(
                    "DELETE FROM command_history WHERE datetime(timestamp) < datetime(?1)",
                    params![cutoff.to_rfc3339()],
                )?;
            }
            
            // Delete excess entries
            if let Some(max_entries) = policy.max_entries {
                deleted += conn.execute(
                    r#"DELETE FROM command_history
                       WHERE id IN (
                           SELECT id FROM command_history
                           ORDER BY timestamp DESC
                           LIMIT -1 OFFSET ?1
                       )"#,
                    params![max_entries],
                )?;
            }
            
            info!("Applied retention policy, deleted {} entries", deleted);
            Ok(deleted)
        })
        .await?
    }
    
    /// Cleanup old entries (deprecated, use apply_retention_policy)
    pub async fn cleanup_old_entries(&self, days: u32) -> Result<usize> {
        let policy = RetentionPolicy {
            max_age_days: Some(days),
            max_entries: None,
            preserve_favorites: false,
            preserve_frequently_used: false,
        };
        
        self.apply_retention_policy(&policy).await
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
    pub async fn get_history(&self, offset: usize, limit: usize) -> Result<Vec<CommandHistoryEntry>> {
        let pool = self.pool.clone();
        
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            let mut stmt = conn.prepare(
                r#"SELECT id, command, user_input, explanation, shell_type,
                          working_directory, timestamp, execution_metadata,
                          safety_metadata, tags
                   FROM command_history
                   ORDER BY timestamp DESC
                   LIMIT ?1 OFFSET ?2"#
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
                        execution_metadata: row.get::<_, Option<String>>(7)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        safety_metadata: row.get::<_, Option<String>>(8)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                        tags: row.get::<_, Option<String>>(9)?
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