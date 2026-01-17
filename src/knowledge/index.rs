//! Knowledge Index using LanceDB
//!
//! Provides persistent storage and semantic search for command knowledge.

use crate::knowledge::{
    schema::{EntryBuilder, EntryType},
    Embedder, KnowledgeError, Result,
};
use arrow_array::{RecordBatch, RecordBatchIterator};
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

/// A knowledge entry retrieved from the index
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    /// The original natural language request
    pub request: String,
    /// The shell command
    pub command: String,
    /// Directory/project context (if available)
    pub context: Option<String>,
    /// Similarity score (0.0 to 1.0, higher is more similar)
    pub similarity: f32,
    /// When this entry was created
    pub timestamp: DateTime<Utc>,
    /// Type of entry (success or correction)
    pub entry_type: EntryType,
    /// For corrections: the original command that was wrong
    pub original_command: Option<String>,
    /// For corrections: feedback about why it was wrong
    pub feedback: Option<String>,
}

/// Local knowledge index for storing and searching command patterns
pub struct KnowledgeIndex {
    db: Connection,
    table: Arc<RwLock<Option<Table>>>,
    embedder: Embedder,
}

impl KnowledgeIndex {
    /// Open or create a knowledge index at the given path
    pub async fn open(path: &Path) -> Result<Self> {
        // Create directory if needed
        std::fs::create_dir_all(path)?;

        // Initialize embedder with cache in same directory
        let cache_dir = path.join("models");
        std::fs::create_dir_all(&cache_dir)?;
        let embedder = Embedder::new(Some(&cache_dir))?;

        // Connect to LanceDB
        let db_path = path.join("vectors.lance");
        let db = connect(db_path.to_str().unwrap())
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

    /// Record a successful command execution
    pub async fn record_success(
        &self,
        request: &str,
        command: &str,
        context: Option<&str>,
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
        );

        self.add_batch(builder.build()?).await
    }

    /// Record a correction from agentic refinement
    pub async fn record_correction(
        &self,
        request: &str,
        original: &str,
        corrected: &str,
        feedback: Option<&str>,
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
        );

        self.add_batch(builder.build()?).await
    }

    /// Find similar past commands
    pub async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>> {
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

    /// Get statistics about the knowledge index
    pub async fn stats(&self) -> Result<KnowledgeStats> {
        let table = self.table.read().await;
        let table = match table.as_ref() {
            Some(t) => t,
            None => {
                return Ok(KnowledgeStats {
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

        // For now, return total count
        // TODO: Add type-specific counts
        Ok(KnowledgeStats {
            total_entries: count,
            success_count: count,
            correction_count: 0,
        })
    }

    /// Clear all entries from the index
    pub async fn clear(&self) -> Result<()> {
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

    /// Add a batch of entries to the index
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
            });
        }

        Ok(entries)
    }
}

/// Statistics about the knowledge index
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    pub total_entries: usize,
    pub success_count: usize,
    pub correction_count: usize,
}

// Tests for this module have been moved to tests/knowledge_integration.rs
// Run with: cargo test --test knowledge_integration --features knowledge -- --ignored
