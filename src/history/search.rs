//! Search functionality for command history
//! 
//! Provides full-text and semantic search capabilities with
//! performance optimizations for constitutional compliance.

use super::models::{CommandHistoryEntry, ExecutionMetadata, SafetyMetadata};
use crate::models::ShellType;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, warn};

/// Search query for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search text
    pub text: String,
    
    /// Optional filters to apply
    pub filters: Option<SearchFilters>,
    
    /// Similarity threshold for semantic search
    pub similarity_threshold: Option<f64>,
    
    /// Maximum results to return
    pub max_results: Option<usize>,
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            filters: None,
            similarity_threshold: None,
            max_results: None,
        }
    }
    
    /// Add filters to the query
    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = Some(filters);
        self
    }
    
    /// Set similarity threshold for semantic search
    pub fn with_similarity_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = Some(threshold);
        self
    }
    
    /// Set maximum results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }
}

/// Search filters for refining results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    
    /// Filter by working directory
    pub working_directory: Option<String>,
    
    /// Filter by shell type
    pub shell_type: Option<String>,
    
    /// Filter by date range
    pub date_range: Option<DateRange>,
    
    /// Filter by risk level
    pub risk_level: Option<String>,
}

impl SearchFilters {
    /// Create new search filters
    pub fn new() -> Self {
        Self {
            tags: None,
            working_directory: None,
            shell_type: None,
            date_range: None,
            risk_level: None,
        }
    }
    
    /// Filter by tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
    
    /// Filter by working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }
    
    /// Filter by shell type
    pub fn with_shell_type(mut self, shell: impl Into<String>) -> Self {
        self.shell_type = Some(shell.into());
        self
    }
    
    /// Filter by date range
    pub fn with_date_range(mut self, range: DateRange) -> Self {
        self.date_range = Some(range);
        self
    }
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self::new()
    }
}

/// Date range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive)
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    
    /// End date (inclusive)
    pub to: Option<chrono::DateTime<chrono::Utc>>,
}

impl DateRange {
    /// Create a date range
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
        }
    }
    
    /// Set start date
    pub fn from(mut self, date: chrono::DateTime<chrono::Utc>) -> Self {
        self.from = Some(date);
        self
    }
    
    /// Set end date
    pub fn to(mut self, date: chrono::DateTime<chrono::Utc>) -> Self {
        self.to = Some(date);
        self
    }
    
    /// Create a range for today
    pub fn today() -> Self {
        let today = chrono::Utc::now().date_naive();
        let start = today.and_hms_opt(0, 0, 0).unwrap();
        let end = today.and_hms_opt(23, 59, 59).unwrap();
        
        Self {
            from: Some(start.and_utc()),
            to: Some(end.and_utc()),
        }
    }
    
    /// Create a range for last N days
    pub fn last_days(days: i64) -> Self {
        let now = chrono::Utc::now();
        let start = now - chrono::Duration::days(days);
        
        Self {
            from: Some(start),
            to: Some(now),
        }
    }
}

impl Default for DateRange {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching history entry
    pub entry: CommandHistoryEntry,
    
    /// Similarity score (0.0 to 1.0)
    pub similarity_score: f64,
    
    /// Context of the match
    pub matching_context: String,
    
    /// Explanation of why this matched
    pub explanation: String,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(entry: CommandHistoryEntry, score: f64) -> Self {
        Self {
            entry,
            similarity_score: score,
            matching_context: String::new(),
            explanation: String::new(),
        }
    }
    
    /// Add matching context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.matching_context = context.into();
        self
    }
    
    /// Add explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (score: {:.2}): {}",
            self.entry.timestamp.format("%Y-%m-%d %H:%M"),
            self.similarity_score,
            self.entry.command
        )
    }
}

/// FTS5 search engine for high-performance full-text search
#[derive(Debug, Clone)]
pub struct FTS5SearchEngine {
    /// Database connection pool
    pool: SqlitePool,
    
    /// Search configuration
    config: FTS5Config,
    
    /// Performance metrics
    metrics: SearchMetrics,
}

impl FTS5SearchEngine {
    /// Create a new FTS5 search engine
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        let config = FTS5Config::default();
        let metrics = SearchMetrics::new();
        
        // Initialize FTS5 virtual table if not exists
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(
                command,
                natural_language,
                output,
                tags,
                working_directory,
                content=history,
                content_rowid=id,
                tokenize='porter unicode61 remove_diacritics 2'
            )
            "#
        )
        .execute(&pool)
        .await?;
        
        // Create triggers to keep FTS5 table in sync
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS history_fts_insert
            AFTER INSERT ON history
            BEGIN
                INSERT INTO history_fts(
                    rowid, command, natural_language, output, tags, working_directory
                )
                VALUES(
                    new.id, new.command, new.natural_language, new.output,
                    new.tags, new.working_directory
                );
            END
            "#
        )
        .execute(&pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS history_fts_delete
            AFTER DELETE ON history
            BEGIN
                DELETE FROM history_fts WHERE rowid = old.id;
            END
            "#
        )
        .execute(&pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS history_fts_update
            AFTER UPDATE ON history
            BEGIN
                DELETE FROM history_fts WHERE rowid = old.id;
                INSERT INTO history_fts(
                    rowid, command, natural_language, output, tags, working_directory
                )
                VALUES(
                    new.id, new.command, new.natural_language, new.output,
                    new.tags, new.working_directory
                );
            END
            "#
        )
        .execute(&pool)
        .await?;
        
        Ok(Self {
            pool,
            config,
            metrics,
        })
    }
    
    /// Perform full-text search
    pub async fn search(&mut self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let start = std::time::Instant::now();
        
        // Build FTS5 query
        let fts_query = self.build_fts_query(&query.text)?;
        
        // Apply filters
        let mut sql = String::from(
            r#"
            SELECT 
                h.*, 
                rank,
                snippet(history_fts, 0, '<match>', '</match>', '...', 32) as command_snippet,
                snippet(history_fts, 1, '<match>', '</match>', '...', 32) as nl_snippet
            FROM history h
            JOIN history_fts ON h.id = history_fts.rowid
            WHERE history_fts MATCH ?
            "#
        );
        
        let mut conditions = Vec::new();
        
        if let Some(filters) = &query.filters {
            if let Some(tags) = &filters.tags {
                let tag_conditions: Vec<String> = tags
                    .iter()
                    .map(|_| "h.tags LIKE ?")
                    .collect();
                if !tag_conditions.is_empty() {
                    conditions.push(format!("({})", tag_conditions.join(" OR ")));
                }
            }
            
            if let Some(dir) = &filters.working_directory {
                conditions.push("h.working_directory = ?".to_string());
            }
            
            if let Some(shell) = &filters.shell_type {
                conditions.push("h.shell_type = ?".to_string());
            }
            
            if let Some(risk) = &filters.risk_level {
                conditions.push("h.risk_level = ?".to_string());
            }
            
            if let Some(range) = &filters.date_range {
                if range.from.is_some() {
                    conditions.push("h.timestamp >= ?".to_string());
                }
                if range.to.is_some() {
                    conditions.push("h.timestamp <= ?".to_string());
                }
            }
        }
        
        if !conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&conditions.join(" AND "));
        }
        
        sql.push_str(" ORDER BY rank DESC");
        
        if let Some(max) = query.max_results {
            sql.push_str(&format!(" LIMIT {}", max));
        } else {
            sql.push_str(" LIMIT 100");
        }
        
        // Build and execute query
        let mut query_builder = sqlx::query_as::<_, FTS5SearchRow>(&sql)
            .bind(&fts_query);
        
        if let Some(filters) = &query.filters {
            if let Some(tags) = &filters.tags {
                for tag in tags {
                    query_builder = query_builder.bind(format!("%{}%", tag));
                }
            }
            
            if let Some(dir) = &filters.working_directory {
                query_builder = query_builder.bind(dir);
            }
            
            if let Some(shell) = &filters.shell_type {
                query_builder = query_builder.bind(shell);
            }
            
            if let Some(risk) = &filters.risk_level {
                query_builder = query_builder.bind(risk);
            }
            
            if let Some(range) = &filters.date_range {
                if let Some(from) = &range.from {
                    query_builder = query_builder.bind(from);
                }
                if let Some(to) = &range.to {
                    query_builder = query_builder.bind(to);
                }
            }
        }
        
        let rows = query_builder.fetch_all(&self.pool).await?;
        
        // Convert to search results
        let mut results = Vec::new();
        for row in rows {
            let entry = CommandHistoryEntry {
                id: row.id.to_string(),
                command: row.command,
                explanation: row.natural_language.clone(),
                shell_type: row.shell_type.parse().unwrap_or(ShellType::Bash),
                working_directory: row.working_directory,
                timestamp: row.timestamp,
                exit_code: row.exit_code,
                safety_assessment: crate::models::SafetyAssessment {
                    is_safe: row.risk_level == "safe",
                    risk_level: row.risk_level.parse().unwrap_or(crate::models::RiskLevel::Low),
                    warnings: Vec::new(),
                    requires_confirmation: false,
                },
                metadata: Some(crate::models::CommandMetadata {
                    tags: row.tags.map(|t| t.split(',').map(String::from).collect()).unwrap_or_default(),
                    confidence: 0.0,
                    model_used: String::new(),
                    generation_time_ms: 0,
                }),
                execution_metadata: Some(ExecutionMetadata {
                    exit_code: row.exit_code,
                    execution_time: row.execution_time_ms.map(|ms| std::time::Duration::from_millis(ms as u64)),
                    output_size: row.output.as_ref().map(|o| o.len()),
                    backend_used: String::new(),
                    generation_time: std::time::Duration::from_millis(0),
                    validation_time: std::time::Duration::from_millis(0),
                    model_name: String::new(),
                    confidence_score: 0.0,
                }),
                safety_metadata: Some(SafetyMetadata {
                    validation_timestamp: chrono::Utc::now(),
                    validation_version: "1.0".to_string(),
                    detected_patterns: Vec::new(),
                    risk_score: 0.0,
                    override_reason: None,
                    validator_id: String::new(),
                }),
                user_context: None,
            };
            
            let similarity_score = self.normalize_rank(row.rank);
            
            let result = SearchResult {
                entry,
                similarity_score,
                matching_context: row.command_snippet.unwrap_or_default(),
                explanation: self.generate_explanation(&row)?,
            };
            
            results.push(result);
        }
        
        // Apply similarity threshold filter
        if let Some(threshold) = query.similarity_threshold {
            results.retain(|r| r.similarity_score >= threshold);
        }
        
        // Update metrics
        self.metrics.record_search(start.elapsed(), results.len());
        
        debug!(
            "FTS5 search completed: query='{}', results={}, time={:?}",
            query.text,
            results.len(),
            start.elapsed()
        );
        
        Ok(results)
    }
    
    /// Build FTS5 query from user input
    fn build_fts_query(&self, text: &str) -> Result<String> {
        let mut query = String::new();
        
        // Parse search operators
        let tokens: Vec<&str> = text.split_whitespace().collect();
        
        for (i, token) in tokens.iter().enumerate() {
            if token.starts_with('+') {
                // Required term
                query.push_str(&format!("\"{}\" ", &token[1..]));
            } else if token.starts_with('-') {
                // Excluded term
                query.push_str(&format!("NOT {} ", &token[1..]));
            } else if token.starts_with('"') && token.ends_with('"') {
                // Exact phrase
                query.push_str(&format!("{} ", token));
            } else if token.contains('*') {
                // Wildcard
                query.push_str(&format!("{} ", token));
            } else if token.to_uppercase() == "AND" || token.to_uppercase() == "OR" || token.to_uppercase() == "NOT" {
                // Boolean operators
                query.push_str(&format!("{} ", token.to_uppercase()));
            } else {
                // Regular term with stemming
                if i > 0 {
                    query.push_str("AND ");
                }
                query.push_str(&format!("{} ", token));
            }
        }
        
        Ok(query.trim().to_string())
    }
    
    /// Normalize FTS5 rank to 0-1 similarity score
    fn normalize_rank(&self, rank: f64) -> f64 {
        // FTS5 rank is negative, closer to 0 is better
        let normalized = 1.0 / (1.0 + rank.abs());
        normalized.clamp(0.0, 1.0)
    }
    
    /// Generate explanation for search result
    fn generate_explanation(&self, row: &FTS5SearchRow) -> Result<String> {
        let mut explanation = Vec::new();
        
        if row.command_snippet.is_some() {
            explanation.push("Matched in command");
        }
        
        if row.nl_snippet.is_some() {
            explanation.push("Matched in natural language description");
        }
        
        if row.risk_level == "high" || row.risk_level == "critical" {
            explanation.push("High risk command");
        }
        
        if row.tags.is_some() {
            explanation.push("Tagged entry");
        }
        
        Ok(explanation.join(", "))
    }
    
    /// Rebuild FTS5 index
    pub async fn rebuild_index(&self) -> Result<()> {
        sqlx::query("INSERT INTO history_fts(history_fts) VALUES('rebuild')")
            .execute(&self.pool)
            .await?;
        
        debug!("FTS5 index rebuilt successfully");
        Ok(())
    }
    
    /// Optimize FTS5 index
    pub async fn optimize_index(&self) -> Result<()> {
        sqlx::query("INSERT INTO history_fts(history_fts) VALUES('optimize')")
            .execute(&self.pool)
            .await?;
        
        debug!("FTS5 index optimized successfully");
        Ok(())
    }
    
    /// Get search suggestions based on partial input
    pub async fn get_suggestions(&self, partial: &str, limit: usize) -> Result<Vec<String>> {
        let query = format!("{}*", partial);
        
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT command
            FROM history_fts
            WHERE history_fts MATCH ?
            ORDER BY rank
            LIMIT ?
            "#
        )
        .bind(&query)
        .bind(limit as i32)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }
}

/// FTS5 configuration
#[derive(Debug, Clone)]
pub struct FTS5Config {
    /// Enable Porter stemming
    pub porter_stemming: bool,
    
    /// Enable Unicode normalization
    pub unicode_normalization: bool,
    
    /// Remove diacritics
    pub remove_diacritics: bool,
    
    /// Column weights for ranking
    pub column_weights: HashMap<String, f64>,
    
    /// Snippet configuration
    pub snippet_length: usize,
    
    /// Highlight markers
    pub highlight_start: String,
    pub highlight_end: String,
}

impl Default for FTS5Config {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("command".to_string(), 2.0);
        weights.insert("natural_language".to_string(), 1.5);
        weights.insert("tags".to_string(), 1.2);
        weights.insert("output".to_string(), 0.8);
        weights.insert("working_directory".to_string(), 0.5);
        
        Self {
            porter_stemming: true,
            unicode_normalization: true,
            remove_diacritics: true,
            column_weights: weights,
            snippet_length: 32,
            highlight_start: "<match>".to_string(),
            highlight_end: "</match>".to_string(),
        }
    }
}

/// FTS5 search result row
#[derive(Debug, Clone, FromRow)]
struct FTS5SearchRow {
    // History fields
    id: i64,
    command: String,
    natural_language: String,
    output: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
    working_directory: String,
    shell_type: String,
    exit_code: Option<i32>,
    execution_time_ms: Option<i64>,
    risk_level: String,
    tags: Option<String>,
    
    // FTS5 fields
    rank: f64,
    command_snippet: Option<String>,
    nl_snippet: Option<String>,
}

/// Search performance metrics
#[derive(Debug, Clone)]
pub struct SearchMetrics {
    /// Total searches performed
    pub total_searches: usize,
    
    /// Average search time
    pub avg_search_time: std::time::Duration,
    
    /// Total results returned
    pub total_results: usize,
    
    /// Cache hit rate
    pub cache_hit_rate: f64,
    
    /// Last optimization time
    pub last_optimization: Option<chrono::DateTime<chrono::Utc>>,
}

impl SearchMetrics {
    pub fn new() -> Self {
        Self {
            total_searches: 0,
            avg_search_time: std::time::Duration::ZERO,
            total_results: 0,
            cache_hit_rate: 0.0,
            last_optimization: None,
        }
    }
    
    pub fn record_search(&mut self, duration: std::time::Duration, results: usize) {
        self.total_searches += 1;
        self.total_results += results;
        
        // Update average search time
        let total_time = self.avg_search_time * self.total_searches as u32;
        self.avg_search_time = (total_time + duration) / (self.total_searches as u32);
    }
}

/// Hybrid search engine combining FTS5 and semantic search
#[derive(Debug)]
pub struct HybridSearchEngine {
    /// FTS5 search engine
    fts_engine: FTS5SearchEngine,
    
    /// Semantic search cache
    semantic_cache: Option<crate::semantic::cache::LocalEmbeddingCache>,
    
    /// Hybrid search configuration
    config: HybridSearchConfig,
}

impl HybridSearchEngine {
    /// Create a new hybrid search engine
    pub async fn new(
        pool: SqlitePool,
        semantic_cache: Option<crate::semantic::cache::LocalEmbeddingCache>,
    ) -> Result<Self> {
        let fts_engine = FTS5SearchEngine::new(pool).await?;
        let config = HybridSearchConfig::default();
        
        Ok(Self {
            fts_engine,
            semantic_cache,
            config,
        })
    }
    
    /// Perform hybrid search combining FTS5 and semantic search
    pub async fn search(&mut self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let start = std::time::Instant::now();
        
        // Perform FTS5 search
        let mut fts_results = self.fts_engine.search(query).await?;
        
        // Perform semantic search if available
        if let Some(cache) = &self.semantic_cache {
            let semantic_results = self.semantic_search(cache, query).await?;
            
            // Merge and re-rank results
            fts_results = self.merge_results(fts_results, semantic_results)?;
        }
        
        debug!(
            "Hybrid search completed: results={}, time={:?}",
            fts_results.len(),
            start.elapsed()
        );
        
        Ok(fts_results)
    }
    
    /// Perform semantic search
    async fn semantic_search(
        &self,
        cache: &crate::semantic::cache::LocalEmbeddingCache,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        // This would integrate with the LocalEmbeddingCache
        // For now, return empty results
        Ok(Vec::new())
    }
    
    /// Merge FTS5 and semantic results
    fn merge_results(
        &self,
        fts_results: Vec<SearchResult>,
        semantic_results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        let mut merged = HashMap::new();
        
        // Add FTS results with weight
        for result in fts_results {
            let score = result.similarity_score * self.config.fts_weight;
            let entry_id = result.entry.id.clone();
            merged.insert(entry_id, (result, score));
        }
        
        // Add or update with semantic results
        for result in semantic_results {
            let score = result.similarity_score * self.config.semantic_weight;
            let entry_id = result.entry.id.clone();
            merged
                .entry(entry_id)
                .and_modify(|(_, s)| *s = (*s + score) / 2.0)
                .or_insert((result, score));
        }
        
        // Sort by combined score
        let mut results: Vec<SearchResult> = merged
            .into_iter()
            .map(|(_, (mut result, score))| {
                result.similarity_score = score;
                result
            })
            .collect();
        
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(results)
    }
    
    /// Rebuild all search indexes
    pub async fn rebuild_indexes(&mut self) -> Result<()> {
        self.fts_engine.rebuild_index().await?;
        
        if let Some(cache) = &mut self.semantic_cache {
            cache.clear_cache().await?;
        }
        
        Ok(())
    }
    
    /// Optimize all search indexes
    pub async fn optimize_indexes(&mut self) -> Result<()> {
        self.fts_engine.optimize_index().await?;
        
        if let Some(cache) = &mut self.semantic_cache {
            cache.cleanup().await?;
        }
        
        Ok(())
    }
}

/// Hybrid search configuration
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    /// Weight for FTS5 results (0.0 - 1.0)
    pub fts_weight: f64,
    
    /// Weight for semantic results (0.0 - 1.0)
    pub semantic_weight: f64,
    
    /// Enable result deduplication
    pub deduplicate: bool,
    
    /// Maximum results to return
    pub max_results: usize,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            fts_weight: 0.6,
            semantic_weight: 0.4,
            deduplicate: true,
            max_results: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("find rust files")
            .with_similarity_threshold(0.8)
            .with_max_results(10);
        
        assert_eq!(query.text, "find rust files");
        assert_eq!(query.similarity_threshold, Some(0.8));
        assert_eq!(query.max_results, Some(10));
    }
    
    #[test]
    fn test_search_filters() {
        let filters = SearchFilters::new()
            .with_tags(vec!["rust".to_string(), "search".to_string()])
            .with_working_directory("/home/user/project");
        
        assert_eq!(filters.tags.unwrap().len(), 2);
        assert_eq!(filters.working_directory.unwrap(), "/home/user/project");
    }
    
    #[test]
    fn test_date_range() {
        let range = DateRange::last_days(7);
        
        assert!(range.from.is_some());
        assert!(range.to.is_some());
        assert!(range.from.unwrap() < range.to.unwrap());
    }
    
    #[test]
    fn test_fts5_config() {
        let config = FTS5Config::default();
        
        assert!(config.porter_stemming);
        assert!(config.unicode_normalization);
        assert!(config.remove_diacritics);
        assert_eq!(config.snippet_length, 32);
        assert_eq!(config.column_weights.get("command"), Some(&2.0));
        assert_eq!(config.column_weights.get("natural_language"), Some(&1.5));
    }
    
    #[test]
    fn test_search_metrics() {
        let mut metrics = SearchMetrics::new();
        
        // Record searches
        metrics.record_search(std::time::Duration::from_millis(100), 10);
        metrics.record_search(std::time::Duration::from_millis(200), 20);
        
        assert_eq!(metrics.total_searches, 2);
        assert_eq!(metrics.total_results, 30);
        assert_eq!(metrics.avg_search_time, std::time::Duration::from_millis(150));
    }
    
    #[test]
    fn test_hybrid_config() {
        let config = HybridSearchConfig::default();
        
        assert_eq!(config.fts_weight, 0.6);
        assert_eq!(config.semantic_weight, 0.4);
        assert!(config.deduplicate);
        assert_eq!(config.max_results, 100);
    }
    
    #[tokio::test]
    async fn test_fts5_engine_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        // Create pool
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Create history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                natural_language TEXT NOT NULL,
                output TEXT,
                timestamp DATETIME NOT NULL,
                working_directory TEXT NOT NULL,
                shell_type TEXT NOT NULL,
                exit_code INTEGER,
                execution_time_ms INTEGER,
                risk_level TEXT NOT NULL,
                tags TEXT,
                favorite BOOLEAN NOT NULL DEFAULT FALSE,
                frequency INTEGER NOT NULL DEFAULT 1
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // Initialize FTS5 engine
        let engine = FTS5SearchEngine::new(pool.clone()).await;
        assert!(engine.is_ok());
        
        // Verify FTS5 table exists
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='history_fts'")
            .fetch_optional(&pool)
            .await
            .unwrap();
        
        assert!(result.is_some());
    }
}