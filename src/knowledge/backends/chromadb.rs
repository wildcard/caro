//! ChromaDB vector backend implementation
//!
//! Provides a server-based vector database using ChromaDB.
//! Requires an external ChromaDB server (local or cloud).
//!
//! Use cases:
//! - Team knowledge sharing
//! - Cloud deployments
//! - CI/CD pre-indexing
//! - Cross-machine sync

use super::{BackendStats, VectorBackend};
use crate::knowledge::{KnowledgeEntry, KnowledgeError, Result};
use crate::models::ChromaDbConfig;
use async_trait::async_trait;
use chroma::client::{ChromaHttpClient, ChromaHttpClientOptions};
use chroma::collection::ChromaCollection;
use chroma::types::{Document, Embedding, Metadata};
use std::collections::HashMap;

const COLLECTION_NAME: &str = "caro_commands";

/// ChromaDB-based vector backend (server-based)
pub struct ChromaDbBackend {
    client: ChromaHttpClient,
    collection: ChromaCollection,
    embedder: crate::knowledge::Embedder,
}

impl ChromaDbBackend {
    /// Create a new ChromaDB backend with the given configuration
    ///
    /// # Arguments
    /// * `config` - ChromaDB server configuration
    /// * `embedder` - Embedder for generating vectors (shared with LanceDB)
    ///
    /// # Returns
    /// A configured ChromaDB backend ready for use
    ///
    /// # Errors
    /// Returns error if:
    /// - Cannot connect to ChromaDB server
    /// - Cannot create/access collection
    pub async fn new(
        config: ChromaDbConfig,
        embedder: crate::knowledge::Embedder,
    ) -> Result<Self> {
        // Build client options
        let mut options = ChromaHttpClientOptions::new(config.url.clone());

        if let Some(api_key) = config.api_key {
            options = options.with_api_key(api_key);
        }

        if let Some(tenant) = config.tenant {
            options = options.with_tenant(tenant);
        }

        if let Some(database) = config.database {
            options = options.with_database(database);
        }

        // Create HTTP client
        let client = ChromaHttpClient::new(options)
            .map_err(|e| KnowledgeError::Database(format!("Failed to create ChromaDB client: {}", e)))?;

        // Get or create collection
        let collection = match client.get_collection(COLLECTION_NAME).await {
            Ok(coll) => coll,
            Err(_) => {
                // Collection doesn't exist, create it
                client
                    .create_collection(COLLECTION_NAME, None)
                    .await
                    .map_err(|e| {
                        KnowledgeError::Database(format!("Failed to create collection: {}", e))
                    })?
            }
        };

        Ok(Self {
            client,
            collection,
            embedder,
        })
    }

    /// Helper to add documents to the collection
    async fn add_documents(
        &self,
        ids: Vec<String>,
        documents: Vec<String>,
        embeddings: Vec<Vec<f32>>,
        metadatas: Vec<HashMap<String, String>>,
    ) -> Result<()> {
        self.collection
            .add(ids, Some(embeddings), Some(metadatas), Some(documents))
            .await
            .map_err(|e| KnowledgeError::Database(format!("Failed to add documents: {}", e)))?;
        Ok(())
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
        let embedding = self.embedder.embed_command(request, command)?;
        let id = uuid::Uuid::new_v4().to_string();

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("request".to_string(), request.to_string());
        metadata.insert("command".to_string(), command.to_string());
        metadata.insert("entry_type".to_string(), "success".to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().timestamp().to_string());

        if let Some(ctx) = context {
            metadata.insert("context".to_string(), ctx.to_string());
        }

        // Document is a combination of request and command for better search
        let document = format!("{} -> {}", request, command);

        self.add_documents(
            vec![id],
            vec![document],
            vec![embedding],
            vec![metadata],
        )
        .await
    }

    async fn record_correction(
        &self,
        request: &str,
        original: &str,
        corrected: &str,
        feedback: Option<&str>,
    ) -> Result<()> {
        let embedding = self.embedder.embed_command(request, corrected)?;
        let id = uuid::Uuid::new_v4().to_string();

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("request".to_string(), request.to_string());
        metadata.insert("command".to_string(), corrected.to_string());
        metadata.insert("entry_type".to_string(), "correction".to_string());
        metadata.insert("original_command".to_string(), original.to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().timestamp().to_string());

        if let Some(fb) = feedback {
            metadata.insert("feedback".to_string(), fb.to_string());
        }

        // Document includes the correction context
        let document = format!("{} -> {} (corrected from: {})", request, corrected, original);

        self.add_documents(
            vec![id],
            vec![document],
            vec![embedding],
            vec![metadata],
        )
        .await
    }

    async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>> {
        // Generate query embedding
        let query_embedding = self.embedder.embed_one(query)?;

        // Perform vector search
        let results = self
            .collection
            .query(
                Some(vec![query_embedding]),
                limit as u32,
                None, // where filter
                None, // where_document filter
                None, // include (default: everything)
            )
            .await
            .map_err(|e| KnowledgeError::Database(format!("Query failed: {}", e)))?;

        // Parse results into KnowledgeEntry
        let mut entries = Vec::new();

        if let Some(metadatas) = results.metadatas {
            for (i, metadata_opt) in metadatas.into_iter().enumerate() {
                if let Some(metadata) = metadata_opt {
                    let request = metadata.get("request").cloned().unwrap_or_default();
                    let command = metadata.get("command").cloned().unwrap_or_default();
                    let context = metadata.get("context").cloned();
                    let entry_type_str = metadata.get("entry_type").cloned().unwrap_or_else(|| "success".to_string());
                    let timestamp_str = metadata.get("timestamp").cloned().unwrap_or_else(|| "0".to_string());
                    let original_command = metadata.get("original_command").cloned();
                    let feedback = metadata.get("feedback").cloned();

                    // Parse entry type
                    let entry_type = crate::knowledge::schema::EntryType::parse(&entry_type_str)
                        .unwrap_or(crate::knowledge::schema::EntryType::Success);

                    // Parse timestamp
                    let timestamp = timestamp_str
                        .parse::<i64>()
                        .ok()
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        .unwrap_or_else(chrono::Utc::now);

                    // Get similarity score (distance)
                    // ChromaDB returns L2 distance, convert to similarity
                    let similarity = if let Some(distances) = &results.distances {
                        if let Some(Some(dist)) = distances.get(i) {
                            1.0 / (1.0 + dist)
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

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
            }
        }

        Ok(entries)
    }

    async fn stats(&self) -> Result<BackendStats> {
        // Get collection count
        let count = self
            .collection
            .count()
            .await
            .map_err(|e| KnowledgeError::Database(format!("Failed to get count: {}", e)))?;

        // TODO: Query with filters to get type-specific counts
        // For now, return total count
        Ok(BackendStats {
            total_entries: count as usize,
            success_count: count as usize, // Approximation
            correction_count: 0,           // Approximation
        })
    }

    async fn clear(&self) -> Result<()> {
        // Delete the collection
        self.client
            .delete_collection(COLLECTION_NAME)
            .await
            .map_err(|e| KnowledgeError::Database(format!("Failed to delete collection: {}", e)))?;

        // Recreate empty collection
        self.client
            .create_collection(COLLECTION_NAME, None)
            .await
            .map_err(|e| KnowledgeError::Database(format!("Failed to recreate collection: {}", e)))?;

        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Try to get collection info as a health check
        self.collection.count().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // Note: These tests require a running ChromaDB server
    // Run with: docker run -p 8000:8000 chromadb/chroma:latest

    #[tokio::test]
    #[ignore = "requires ChromaDB server and model download"]
    async fn test_chromadb_backend_create() {
        let config = ChromaDbConfig::default();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("models");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let embedder = crate::knowledge::Embedder::new(Some(&cache_dir)).unwrap();

        let backend = ChromaDbBackend::new(config, embedder).await.unwrap();
        assert!(backend.is_healthy().await);
        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    #[ignore = "requires ChromaDB server and model download"]
    async fn test_chromadb_record_and_search() {
        let config = ChromaDbConfig::default();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("models");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let embedder = crate::knowledge::Embedder::new(Some(&cache_dir)).unwrap();

        let backend = ChromaDbBackend::new(config, embedder).await.unwrap();

        // Record a success
        backend
            .record_success("list all files", "ls -la", Some("rust project"))
            .await
            .unwrap();

        // Search for similar
        let results = backend.find_similar("show files", 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].command, "ls -la");
    }
}
