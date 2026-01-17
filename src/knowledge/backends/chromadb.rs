//! ChromaDB vector backend implementation
//!
//! Provides a server-based vector database using ChromaDB.
//! Requires a running ChromaDB server (local or remote).

use super::{BackendStats, VectorBackend};
use crate::knowledge::{
    schema::EntryType, Embedder, KnowledgeEntry, KnowledgeError, Result,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use chromadb::v1::client::{ChromaClient, ChromaClientOptions};
use chromadb::v1::collection::{ChromaCollection, CollectionEntries, GetOptions, QueryOptions};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const COLLECTION_NAME: &str = "caro_commands";

/// ChromaDB-based vector backend (server-based)
pub struct ChromaDbBackend {
    client: ChromaClient,
    collection: Arc<RwLock<Option<Collection>>>,
    embedder: Embedder,
}

impl ChromaDbBackend {
    /// Create a new ChromaDB backend connecting to a server
    ///
    /// # Arguments
    /// * `url` - ChromaDB server URL (e.g., "http://localhost:8000")
    /// * `cache_dir` - Optional directory for embedding model cache
    ///
    /// # Returns
    /// A configured ChromaDB backend ready for use
    pub async fn new(url: &str, cache_dir: Option<&std::path::Path>) -> Result<Self> {
        // Initialize embedder with cache
        let embedder = if let Some(dir) = cache_dir {
            std::fs::create_dir_all(dir)?;
            Embedder::new(Some(dir))?
        } else {
            Embedder::new(None)?
        };

        // Connect to ChromaDB server
        let client = ChromaClient::new(chromadb::v2::client::ChromaClientOptions {
            url: url.to_string(),
            ..Default::default()
        });

        // Check if collection exists, create if not
        let collection = match client.get_collection(COLLECTION_NAME).await {
            Ok(coll) => Some(coll),
            Err(_) => {
                // Collection doesn't exist yet, will create on first write
                None
            }
        };

        Ok(Self {
            client,
            collection: Arc::new(RwLock::new(collection)),
            embedder,
        })
    }

    /// Ensure collection exists, creating it if necessary
    async fn ensure_collection(&self) -> Result<()> {
        let mut coll_guard = self.collection.write().await;

        if coll_guard.is_none() {
            let new_collection = self
                .client
                .create_collection(COLLECTION_NAME, None)
                .await
                .map_err(|e| KnowledgeError::Database(e.to_string()))?;
            *coll_guard = Some(new_collection);
        }

        Ok(())
    }

    /// Convert metadata HashMap to ChromaDB metadata format
    fn build_metadata(
        entry_type: EntryType,
        request: &str,
        context: Option<&str>,
        timestamp: DateTime<Utc>,
        original_command: Option<&str>,
        feedback: Option<&str>,
    ) -> HashMap<String, chromadb::v2::collection::MetadataValue> {
        let mut metadata = HashMap::new();

        metadata.insert(
            "entry_type".to_string(),
            chromadb::v2::collection::MetadataValue::Str(entry_type.to_string()),
        );
        metadata.insert(
            "request".to_string(),
            chromadb::v2::collection::MetadataValue::Str(request.to_string()),
        );
        metadata.insert(
            "timestamp".to_string(),
            chromadb::v2::collection::MetadataValue::Int(timestamp.timestamp()),
        );

        if let Some(ctx) = context {
            metadata.insert(
                "context".to_string(),
                chromadb::v2::collection::MetadataValue::Str(ctx.to_string()),
            );
        }

        if let Some(orig) = original_command {
            metadata.insert(
                "original_command".to_string(),
                chromadb::v2::collection::MetadataValue::Str(orig.to_string()),
            );
        }

        if let Some(fb) = feedback {
            metadata.insert(
                "feedback".to_string(),
                chromadb::v2::collection::MetadataValue::Str(fb.to_string()),
            );
        }

        metadata
    }

    /// Parse ChromaDB query results into KnowledgeEntry
    fn parse_results(
        &self,
        ids: Vec<String>,
        documents: Vec<String>,
        metadatas: Vec<HashMap<String, chromadb::v2::collection::MetadataValue>>,
        distances: Vec<f32>,
    ) -> Result<Vec<KnowledgeEntry>> {
        let mut entries = Vec::new();

        for (i, document) in documents.iter().enumerate() {
            let metadata = &metadatas[i];

            let entry_type = metadata
                .get("entry_type")
                .and_then(|v| match v {
                    chromadb::v2::collection::MetadataValue::Str(s) => {
                        EntryType::parse(s).ok()
                    }
                    _ => None,
                })
                .unwrap_or(EntryType::Success);

            let request = metadata
                .get("request")
                .and_then(|v| match v {
                    chromadb::v2::collection::MetadataValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let context = metadata.get("context").and_then(|v| match v {
                chromadb::v2::collection::MetadataValue::Str(s) => Some(s.clone()),
                _ => None,
            });

            let timestamp = metadata
                .get("timestamp")
                .and_then(|v| match v {
                    chromadb::v2::collection::MetadataValue::Int(ts) => {
                        DateTime::from_timestamp(*ts, 0)
                    }
                    _ => None,
                })
                .unwrap_or_else(Utc::now);

            let original_command = metadata.get("original_command").and_then(|v| match v {
                chromadb::v2::collection::MetadataValue::Str(s) => Some(s.clone()),
                _ => None,
            });

            let feedback = metadata.get("feedback").and_then(|v| match v {
                chromadb::v2::collection::MetadataValue::Str(s) => Some(s.clone()),
                _ => None,
            });

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

        let metadata = Self::build_metadata(
            EntryType::Success,
            request,
            context,
            timestamp,
            None,
            None,
        );

        let coll = self.collection.read().await;
        let collection = coll.as_ref().ok_or_else(|| {
            KnowledgeError::Database("Collection not initialized".to_string())
        })?;

        collection
            .add(
                vec![id],
                Some(vec![embedding]),
                Some(vec![metadata]),
                Some(vec![command.to_string()]),
            )
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

        let coll = self.collection.read().await;
        let collection = coll.as_ref().ok_or_else(|| {
            KnowledgeError::Database("Collection not initialized".to_string())
        })?;

        collection
            .add(
                vec![id],
                Some(vec![embedding]),
                Some(vec![metadata]),
                Some(vec![corrected.to_string()]),
            )
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>> {
        let coll = self.collection.read().await;
        let collection = match coll.as_ref() {
            Some(c) => c,
            None => return Ok(vec![]), // No collection yet = no results
        };

        // Generate query embedding
        let query_embedding = self.embedder.embed_one(query)?;

        // Perform vector search
        let results = collection
            .query(
                Some(vec![query_embedding]),
                Some(limit as i32),
                None, // No where clause
                None, // No where_document clause
                None, // Include all by default
            )
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // Parse results - ChromaDB returns batched results
        if results.ids.is_empty() || results.ids[0].is_empty() {
            return Ok(vec![]);
        }

        let ids = results.ids[0].clone();
        let documents = results.documents[0]
            .iter()
            .map(|d| d.clone().unwrap_or_default())
            .collect();
        let metadatas = results.metadatas[0]
            .iter()
            .map(|m| m.clone().unwrap_or_default())
            .collect();
        let distances = results.distances[0].clone();

        self.parse_results(ids, documents, metadatas, distances)
    }

    async fn stats(&self) -> Result<BackendStats> {
        let coll = self.collection.read().await;
        let collection = match coll.as_ref() {
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
        let count_result = collection
            .count()
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        // For now, return total count
        // TODO: Add type-specific counts by querying with metadata filters
        Ok(BackendStats {
            total_entries: count_result as usize,
            success_count: count_result as usize,
            correction_count: 0,
        })
    }

    async fn clear(&self) -> Result<()> {
        let mut coll = self.collection.write().await;
        if coll.is_some() {
            self.client
                .delete_collection(COLLECTION_NAME)
                .await
                .map_err(|e| KnowledgeError::Database(e.to_string()))?;
            *coll = None;
        }
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Check health by attempting to list collections
        self.client.list_collections().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_health() {
        let backend = ChromaDbBackend::new("http://localhost:8000", None)
            .await
            .expect("Failed to create ChromaDB backend");

        assert!(backend.is_healthy().await, "ChromaDB should be healthy");
    }
}
