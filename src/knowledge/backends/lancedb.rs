//! LanceDB vector backend implementation
//!
//! Provides an embedded, local-first vector database using LanceDB.
//! This is the default backend and requires no external services.

use super::{BackendStats, VectorBackend};
use crate::knowledge::{
    collections::{CollectionType, QueryScope},
    schema::{EntryBuilder, EntryType},
    Embedder, KnowledgeEntry, KnowledgeError, Result,
};
use arrow_array::RecordBatch;
use arrow_array::RecordBatchIterator;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
    Connection, Table,
};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

const TABLE_NAME: &str = "commands";

/// LanceDB-based vector backend (embedded, local-first)
pub struct LanceDbBackend {
    db: Connection,
    table: Arc<RwLock<Option<Table>>>,
    embedder: Embedder,
}

impl LanceDbBackend {
    /// Create a new LanceDB backend at the given path
    ///
    /// # Arguments
    /// * `path` - Directory where the LanceDB database will be stored
    ///
    /// # Returns
    /// A configured LanceDB backend ready for use
    pub async fn new(path: &Path) -> Result<Self> {
        // Create directory if needed
        std::fs::create_dir_all(path)?;

        // Initialize embedder with cache in same directory
        let cache_dir = path.join("models");
        std::fs::create_dir_all(&cache_dir)?;
        let embedder = Embedder::new(Some(&cache_dir))?;

        // Connect to LanceDB
        let db_path = path.join("vectors.lance");
        let db_path_str = db_path.to_str().ok_or_else(|| {
            KnowledgeError::Database(format!("Invalid UTF-8 in path: {:?}", db_path))
        })?;
        let db = connect(db_path_str)
            .execute()
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Check if table exists
        let table_names = db
            .table_names()
            .execute()
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        let table = if table_names.contains(&TABLE_NAME.to_string()) {
            let t = db
                .open_table(TABLE_NAME)
                .execute()
                .await
                .map_err(|e| KnowledgeError::Database(e.to_string()))?;
            Some(t)
        } else {
            None
        };

        Ok(Self {
            db,
            table: Arc::new(RwLock::new(table)),
            embedder,
        })
    }

    /// Add a batch of entries to the database
    async fn add_batch(&self, batch: RecordBatch) -> Result<()> {
        let mut table_guard = self.table.write().await;

        match table_guard.as_mut() {
            Some(table) => {
                // Add to existing table
                let schema = batch.schema();
                let iter = RecordBatchIterator::new(vec![Ok(batch)], schema);
                table
                    .add(Box::new(iter))
                    .execute()
                    .await
                    .map_err(|e: lancedb::Error| KnowledgeError::Database(e.to_string()))?;
            }
            None => {
                // Create new table
                let schema = batch.schema();
                let iter = RecordBatchIterator::new(vec![Ok(batch)], schema);
                let new_table = self
                    .db
                    .create_table(TABLE_NAME, Box::new(iter))
                    .execute()
                    .await
                    .map_err(|e| KnowledgeError::Database(e.to_string()))?;
                *table_guard = Some(new_table);
            }
        }

        Ok(())
    }

    /// Parse a record batch into knowledge entries
    fn parse_batch(&self, batch: &RecordBatch) -> Result<Vec<KnowledgeEntry>> {
        use arrow_array::{cast::AsArray, types::Float32Type, types::TimestampSecondType, Array};

        let mut entries = Vec::new();

        let requests = batch
            .column_by_name("request")
            .ok_or_else(|| KnowledgeError::Schema("missing request column".to_string()))?
            .as_string::<i32>();

        let commands = batch
            .column_by_name("command")
            .ok_or_else(|| KnowledgeError::Schema("missing command column".to_string()))?
            .as_string::<i32>();

        let contexts = batch
            .column_by_name("context")
            .ok_or_else(|| KnowledgeError::Schema("missing context column".to_string()))?
            .as_string::<i32>();

        let entry_types = batch
            .column_by_name("entry_type")
            .ok_or_else(|| KnowledgeError::Schema("missing entry_type column".to_string()))?
            .as_string::<i32>();

        let timestamps = batch
            .column_by_name("timestamp")
            .ok_or_else(|| KnowledgeError::Schema("missing timestamp column".to_string()))?
            .as_primitive::<TimestampSecondType>();

        let original_commands = batch
            .column_by_name("original_command")
            .ok_or_else(|| KnowledgeError::Schema("missing original_command column".to_string()))?
            .as_string::<i32>();

        let feedbacks = batch
            .column_by_name("feedback")
            .ok_or_else(|| KnowledgeError::Schema("missing feedback column".to_string()))?
            .as_string::<i32>();

        let profiles = batch
            .column_by_name("profile")
            .ok_or_else(|| KnowledgeError::Schema("missing profile column".to_string()))?
            .as_string::<i32>();

        // Try to get distance column (from vector search)
        let distances = batch
            .column_by_name("_distance")
            .map(|c| c.as_primitive::<Float32Type>());

        for i in 0..batch.num_rows() {
            let request = requests.value(i).to_string();
            let command = commands.value(i).to_string();
            let context = if contexts.is_null(i) {
                None
            } else {
                Some(contexts.value(i).to_string())
            };
            let entry_type = EntryType::parse(entry_types.value(i)).unwrap_or(EntryType::Success);
            let timestamp =
                DateTime::from_timestamp(timestamps.value(i), 0).unwrap_or_else(Utc::now);
            let original_command = if original_commands.is_null(i) {
                None
            } else {
                Some(original_commands.value(i).to_string())
            };
            let feedback = if feedbacks.is_null(i) {
                None
            } else {
                Some(feedbacks.value(i).to_string())
            };

            let profile = if profiles.is_null(i) {
                None
            } else {
                Some(profiles.value(i).to_string())
            };

            // Convert distance to similarity (lower distance = higher similarity)
            let similarity = distances
                .as_ref()
                .map(|d| {
                    if d.is_null(i) {
                        0.0
                    } else {
                        // Convert L2 distance to similarity score
                        1.0 / (1.0 + d.value(i))
                    }
                })
                .unwrap_or(1.0);

            entries.push(KnowledgeEntry {
                request,
                command,
                context,
                similarity,
                timestamp,
                entry_type,
                original_command,
                feedback,
                profile,
            });
        }

        Ok(entries)
    }
}

#[async_trait]
impl VectorBackend for LanceDbBackend {
    async fn record_success(
        &self,
        request: &str,
        command: &str,
        context: Option<&str>,
        profile: Option<&str>,
    ) -> Result<()> {
        let embedding = self.embedder.embed_command(request, command)?;
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let mut builder = EntryBuilder::new();
        builder.add_success(
            id,
            request,
            command,
            context.map(|s| s.to_string()),
            embedding,
            timestamp,
            profile.map(|s| s.to_string()),
        );

        self.add_batch(builder.build()?).await
    }

    async fn record_correction(
        &self,
        request: &str,
        original: &str,
        corrected: &str,
        feedback: Option<&str>,
        profile: Option<&str>,
    ) -> Result<()> {
        let embedding = self.embedder.embed_command(request, corrected)?;
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let mut builder = EntryBuilder::new();
        builder.add_correction(
            id,
            request,
            corrected,
            original,
            feedback.map(|s| s.to_string()),
            embedding,
            timestamp,
            profile.map(|s| s.to_string()),
        );

        self.add_batch(builder.build()?).await
    }

    async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>> {
        let table = self.table.read().await;
        let table = match table.as_ref() {
            Some(t) => t,
            None => return Ok(vec![]), // No table yet = no results
        };

        // Generate query embedding
        let query_embedding = self.embedder.embed_one(query)?;

        // Perform vector search
        let results = table
            .vector_search(query_embedding)
            .map_err(|e| KnowledgeError::Database(e.to_string()))?
            .limit(limit)
            .execute()
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Convert to stream and collect batches
        use futures::StreamExt;
        let mut batches = Vec::new();
        let mut stream = results;
        while let Some(batch_result) = stream.next().await {
            match batch_result {
                Ok(batch) => batches.push(batch),
                Err(e) => return Err(KnowledgeError::Database(e.to_string())),
            }
        }

        // Parse results
        let mut entries = Vec::new();
        for batch in batches {
            entries.extend(self.parse_batch(&batch)?);
        }

        Ok(entries)
    }

    async fn stats(&self) -> Result<BackendStats> {
        let table = self.table.read().await;
        let table = match table.as_ref() {
            Some(t) => t,
            None => {
                return Ok(BackendStats {
                    total_entries: 0,
                    success_count: 0,
                    correction_count: 0,
                })
            }
        };

        let count = table
            .count_rows(None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Count successes using filter query
        let success_results = table
            .query()
            .only_if("entry_type = 'success'")
            .execute()
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Convert stream to count
        use futures::StreamExt;
        let mut success_count = 0;
        let mut stream = success_results;
        while let Some(batch_result) = stream.next().await {
            match batch_result {
                Ok(batch) => success_count += batch.num_rows(),
                Err(e) => return Err(KnowledgeError::Database(e.to_string())),
            }
        }

        // Count corrections using filter query
        let correction_results = table
            .query()
            .only_if("entry_type = 'correction'")
            .execute()
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Convert stream to count
        let mut correction_count = 0;
        let mut stream = correction_results;
        while let Some(batch_result) = stream.next().await {
            match batch_result {
                Ok(batch) => correction_count += batch.num_rows(),
                Err(e) => return Err(KnowledgeError::Database(e.to_string())),
            }
        }

        Ok(BackendStats {
            total_entries: count,
            success_count,
            correction_count,
        })
    }

    async fn clear(&self) -> Result<()> {
        let mut table = self.table.write().await;
        if table.is_some() {
            self.db
                .drop_table(TABLE_NAME, &[])
                .await
                .map_err(|e| KnowledgeError::Database(e.to_string()))?;
            *table = None;
        }
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // LanceDB is embedded, so health is based on whether we can access the database
        // Try to list tables as a health check
        self.db.table_names().execute().await.is_ok()
    }

    async fn add_entry(&self, entry: KnowledgeEntry, _collection: CollectionType) -> Result<()> {
        // TODO: Implement table-per-collection architecture
        // For now, add to the default table regardless of collection
        // This allows Phase 4 indexers to work while we refactor for multi-collection

        // Generate embedding from request and command
        let embedding = self
            .embedder
            .embed_command(&entry.request, &entry.command)?;
        let id = uuid::Uuid::new_v4().to_string();
        let mut builder = EntryBuilder::new();

        match entry.entry_type {
            EntryType::Success => {
                builder.add_success(
                    id,
                    entry.request,
                    entry.command,
                    entry.context,
                    embedding,
                    entry.timestamp,
                    entry.profile,
                );
            }
            EntryType::Correction => {
                // For corrections, we need the original command
                // If not provided, use empty string as placeholder
                let original = entry.original_command.unwrap_or_default();
                builder.add_correction(
                    id,
                    entry.request,
                    entry.command,
                    original,
                    entry.feedback,
                    embedding,
                    entry.timestamp,
                    entry.profile,
                );
            }
        }

        self.add_batch(builder.build()?).await
    }

    async fn find_similar_in(
        &self,
        query: &str,
        limit: usize,
        _scope: QueryScope,
    ) -> Result<Vec<KnowledgeEntry>> {
        // TODO: Implement collection filtering for table-per-collection architecture
        // For now, search across all entries (single table)
        // This allows Phase 4 indexers to work while we refactor for multi-collection

        self.find_similar(query, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_lancedb_backend_create() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LanceDbBackend::new(temp_dir.path()).await.unwrap();
        assert!(backend.is_healthy().await);
        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_lancedb_record_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LanceDbBackend::new(temp_dir.path()).await.unwrap();

        // Record a success
        backend
            .record_success("list all files", "ls -la", Some("rust project"), None)
            .await
            .unwrap();

        // Search for similar
        let results = backend.find_similar("show files", 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].command, "ls -la");
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_lancedb_record_correction() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LanceDbBackend::new(temp_dir.path()).await.unwrap();

        backend
            .record_correction(
                "show disk usage",
                "ls -lh",
                "du -h -d 1",
                Some("ls shows files not disk usage"),
                None,
            )
            .await
            .unwrap();

        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_lancedb_clear() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LanceDbBackend::new(temp_dir.path()).await.unwrap();

        backend
            .record_success("test", "echo test", None, None)
            .await
            .unwrap();

        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);

        backend.clear().await.unwrap();

        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }
}
