//! ChromaDB vector backend implementation
//!
//! Provides a server-based vector database using ChromaDB.
//! Requires a running ChromaDB server (local or remote).

use super::{BackendStats, VectorBackend};
use crate::knowledge::{
    collections::{CollectionType, QueryScope},
    schema::EntryType,
    Embedder, KnowledgeEntry, KnowledgeError, Result,
};
use async_trait::async_trait;
use chromadb::client::{ChromaAuthMethod, ChromaClient, ChromaClientOptions, ChromaTokenHeader};
use chromadb::collection::{ChromaCollection, CollectionEntries, QueryOptions};
use chrono::{DateTime, Utc};
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

const COLLECTION_NAME: &str = "caro_commands";

/// ChromaDB-based vector backend (server-based)
pub struct ChromaDbBackend {
    client: ChromaClient,
    collection: Arc<RwLock<Option<ChromaCollection>>>,
    embedder: Embedder,
}

impl ChromaDbBackend {
    /// Create a new ChromaDB backend connecting to a server
    ///
    /// # Arguments
    /// * `url` - ChromaDB server URL (e.g., "http://localhost:8000")
    /// * `cache_dir` - Optional directory for embedding model cache
    /// * `auth_token` - Optional authentication token for ChromaDB server
    ///
    /// # Returns
    /// A configured ChromaDB backend ready for use
    pub async fn new(
        url: &str,
        cache_dir: Option<&std::path::Path>,
        auth_token: Option<&str>,
    ) -> Result<Self> {
        // Initialize embedder with cache
        let embedder = if let Some(dir) = cache_dir {
            std::fs::create_dir_all(dir)?;
            Embedder::new(Some(dir))?
        } else {
            Embedder::new(None)?
        };

        // Configure authentication
        let auth = if let Some(token) = auth_token {
            ChromaAuthMethod::TokenAuth {
                token: token.to_string(),
                header: ChromaTokenHeader::XChromaToken,
            }
        } else {
            ChromaAuthMethod::None
        };

        // Connect to ChromaDB server
        let client = ChromaClient::new(ChromaClientOptions {
            url: Some(url.to_string()),
            auth,
            ..Default::default()
        })
        .await
        .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Check if collection exists
        let collection = client
            .get_or_create_collection(COLLECTION_NAME, None)
            .await
            .ok();

        Ok(Self {
            client,
            collection: Arc::new(RwLock::new(collection)),
            embedder,
        })
    }

    /// Ensure collection exists, creating it if necessary
    async fn ensure_collection(&self) -> Result<()> {
        let coll_guard = self.collection.read().await;

        if coll_guard.is_some() {
            return Ok(());
        }

        drop(coll_guard);

        let mut coll_guard = self.collection.write().await;

        // Double-check after acquiring write lock
        if coll_guard.is_some() {
            return Ok(());
        }

        let new_collection = self
            .client
            .get_or_create_collection(COLLECTION_NAME, None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        *coll_guard = Some(new_collection);
        Ok(())
    }

    /// Build metadata JSON object
    fn build_metadata(
        entry_type: EntryType,
        request: &str,
        context: Option<&str>,
        timestamp: DateTime<Utc>,
        original_command: Option<&str>,
        feedback: Option<&str>,
    ) -> Map<String, Value> {
        let mut metadata = Map::new();

        metadata.insert("entry_type".to_string(), json!(entry_type.as_str()));
        metadata.insert("request".to_string(), json!(request));
        metadata.insert("timestamp".to_string(), json!(timestamp.timestamp()));

        if let Some(ctx) = context {
            metadata.insert("context".to_string(), json!(ctx));
        }

        if let Some(orig) = original_command {
            metadata.insert("original_command".to_string(), json!(orig));
        }

        if let Some(fb) = feedback {
            metadata.insert("feedback".to_string(), json!(fb));
        }

        metadata
    }

    /// Parse ChromaDB query results into KnowledgeEntry
    fn parse_results(
        _ids: Vec<String>,
        documents: Vec<String>,
        metadatas: Vec<Map<String, Value>>,
        distances: Vec<f32>,
    ) -> Result<Vec<KnowledgeEntry>> {
        let mut entries = Vec::new();

        for (i, document) in documents.iter().enumerate() {
            let metadata = &metadatas[i];

            let entry_type = metadata
                .get("entry_type")
                .and_then(|v| v.as_str())
                .and_then(EntryType::parse)
                .unwrap_or(EntryType::Success);

            let request = metadata
                .get("request")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let context = metadata
                .get("context")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let timestamp = metadata
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .and_then(|ts| DateTime::from_timestamp(ts, 0))
                .unwrap_or_else(Utc::now);

            let original_command = metadata
                .get("original_command")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let feedback = metadata
                .get("feedback")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Convert ChromaDB distance to similarity (lower distance = higher similarity)
            let similarity = 1.0 / (1.0 + distances[i]);

            entries.push(KnowledgeEntry {
                request,
                command: document.clone(),
                context,
                similarity,
                timestamp,
                entry_type,
                original_command,
                feedback,
                profile: None, // TODO: Read profile from metadata
            });
        }

        Ok(entries)
    }
}

#[async_trait]
impl VectorBackend for ChromaDbBackend {
    async fn record_success(
        &self,
        request: &str,
        command: &str,
        context: Option<&str>,
    ) -> Result<()> {
        self.ensure_collection().await?;

        let embedding = self.embedder.embed_command(request, command)?;
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let metadata =
            Self::build_metadata(EntryType::Success, request, context, timestamp, None, None);

        let entries = CollectionEntries {
            ids: vec![&id],
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata]),
            documents: Some(vec![command]),
        };

        let coll_guard = self.collection.read().await;
        let collection = coll_guard.as_ref().ok_or_else(|| {
            KnowledgeError::Database("Collection not initialized after ensure_collection".into())
        })?;

        collection
            .add(entries, None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn record_correction(
        &self,
        request: &str,
        original: &str,
        corrected: &str,
        feedback: Option<&str>,
    ) -> Result<()> {
        self.ensure_collection().await?;

        let embedding = self.embedder.embed_command(request, corrected)?;
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let metadata = Self::build_metadata(
            EntryType::Correction,
            request,
            None,
            timestamp,
            Some(original),
            feedback,
        );

        let entries = CollectionEntries {
            ids: vec![&id],
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata]),
            documents: Some(vec![corrected]),
        };

        let coll_guard = self.collection.read().await;
        let collection = coll_guard.as_ref().ok_or_else(|| {
            KnowledgeError::Database("Collection not initialized after ensure_collection".into())
        })?;

        collection
            .add(entries, None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>> {
        let coll_guard = self.collection.read().await;
        let collection = match coll_guard.as_ref() {
            Some(c) => c,
            None => return Ok(vec![]), // No collection yet = no results
        };

        // Generate query embedding
        let query_embedding = self.embedder.embed_one(query)?;

        // Perform vector search
        let query_options = QueryOptions {
            query_embeddings: Some(vec![query_embedding]),
            n_results: Some(limit),
            query_texts: None,
            where_metadata: None,
            where_document: None,
            include: Some(vec!["documents", "metadatas", "distances"]),
        };

        let results = collection
            .query(query_options, None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Parse results - ChromaDB returns batched results
        if results.ids.is_empty() || results.ids[0].is_empty() {
            return Ok(vec![]);
        }

        let ids = results.ids[0].clone();

        let documents = results
            .documents
            .as_ref()
            .and_then(|docs| docs.first())
            .cloned()
            .unwrap_or_default();

        let metadatas = results
            .metadatas
            .as_ref()
            .and_then(|metas| metas.first())
            .map(|meta_vec| meta_vec.iter().filter_map(|m| m.clone()).collect())
            .unwrap_or_default();

        let distances = results
            .distances
            .as_ref()
            .and_then(|dists| dists.first())
            .cloned()
            .unwrap_or_else(|| vec![0.0; ids.len()]);

        Self::parse_results(ids, documents, metadatas, distances)
    }

    async fn stats(&self) -> Result<BackendStats> {
        let coll_guard = self.collection.read().await;
        let collection = match coll_guard.as_ref() {
            Some(c) => c,
            None => {
                return Ok(BackendStats {
                    total_entries: 0,
                    success_count: 0,
                    correction_count: 0,
                })
            }
        };

        // Get total count
        let count = collection
            .count()
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // For now, return total count
        // TODO: Add type-specific counts by querying with metadata filters
        Ok(BackendStats {
            total_entries: count,
            success_count: count,
            correction_count: 0,
        })
    }

    async fn clear(&self) -> Result<()> {
        let mut coll_guard = self.collection.write().await;
        if coll_guard.is_some() {
            self.client
                .delete_collection(COLLECTION_NAME)
                .await
                .map_err(|e| KnowledgeError::Database(e.to_string()))?;
            *coll_guard = None;
        }
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Check health by attempting to heartbeat
        self.client.heartbeat().await.is_ok()
    }

    async fn add_entry(&self, entry: KnowledgeEntry, _collection: CollectionType) -> Result<()> {
        // TODO: Implement native ChromaDB collection support
        // For now, add to the default collection regardless of collection type
        // This allows Phase 4 indexers to work while we refactor for multi-collection

        self.ensure_collection().await?;

        // Generate embedding from request and command
        let embedding = self
            .embedder
            .embed_command(&entry.request, &entry.command)?;

        // Build metadata
        let metadata = Self::build_metadata(
            entry.entry_type,
            &entry.request,
            entry.context.as_deref(),
            entry.timestamp,
            entry.original_command.as_deref(),
            entry.feedback.as_deref(),
        );

        // Add to ChromaDB
        let id = uuid::Uuid::new_v4().to_string();
        let document = entry.command.clone();

        let entries = CollectionEntries {
            ids: vec![id.as_str()],
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata]),
            documents: Some(vec![document.as_str()]),
        };

        let coll_guard = self.collection.read().await;
        let collection = coll_guard.as_ref().ok_or_else(|| {
            KnowledgeError::Database("Collection not initialized after ensure_collection".into())
        })?;

        collection
            .add(entries, None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_similar_in(
        &self,
        query: &str,
        limit: usize,
        _scope: QueryScope,
    ) -> Result<Vec<KnowledgeEntry>> {
        // TODO: Implement collection filtering for native ChromaDB collections
        // For now, search across all entries (single collection)
        // This allows Phase 4 indexers to work while we refactor for multi-collection

        self.find_similar(query, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_health() {
        let backend = ChromaDbBackend::new("http://localhost:8000", None, None)
            .await
            .expect("Failed to create ChromaDB backend");

        assert!(backend.is_healthy().await, "ChromaDB should be healthy");
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_record_and_search() {
        let backend = ChromaDbBackend::new("http://localhost:8000", None, None)
            .await
            .expect("Failed to create ChromaDB backend");

        // Clear any existing data
        backend.clear().await.expect("Failed to clear collection");

        // Record a success
        backend
            .record_success("list files", "ls -la", Some("/home/user"))
            .await
            .expect("Failed to record success");

        // Search for similar
        let results = backend
            .find_similar("show files", 5)
            .await
            .expect("Failed to search");

        assert!(!results.is_empty(), "Should find similar entries");
        assert_eq!(results[0].command, "ls -la");
    }
}
