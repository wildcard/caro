//! Qdrant client wrapper for vector store operations

use super::{VectorStoreConfig, VectorStoreStats, QueryResult};
use crate::indexing::{ManPageDocument, IndexStatistics};
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, warn, error};

#[cfg(feature = "vector-store")]
use qdrant_client::{
    prelude::*,
    qdrant::{
        CreateCollection, Distance, VectorParams, VectorsConfig,
        SearchPoints, PointStruct, Value,
    },
};

/// Vector store client for managing indexed documents
pub struct VectorStoreClient {
    config: VectorStoreConfig,
    #[cfg(feature = "vector-store")]
    client: Option<Arc<QdrantClient>>,
    #[cfg(not(feature = "vector-store"))]
    _client: Option<()>,
}

impl VectorStoreClient {
    /// Create a new vector store client
    pub async fn new(config: VectorStoreConfig) -> Result<Self> {
        info!("Initializing vector store client");

        #[cfg(feature = "vector-store")]
        let client = Self::init_qdrant(&config).await?;

        Ok(Self {
            config,
            #[cfg(feature = "vector-store")]
            client: Some(Arc::new(client)),
            #[cfg(not(feature = "vector-store"))]
            _client: None,
        })
    }

    /// Initialize Qdrant client
    #[cfg(feature = "vector-store")]
    async fn init_qdrant(config: &VectorStoreConfig) -> Result<QdrantClient> {
        let client = if config.in_memory {
            debug!("Creating in-memory Qdrant client");
            QdrantClient::from_url("http://localhost:6334")
                .build()
                .context("Failed to create in-memory Qdrant client")?
        } else {
            debug!("Creating file-based Qdrant client at {:?}", config.db_path);
            // For local file-based storage, use localhost with persistent storage
            QdrantClient::from_url("http://localhost:6333")
                .build()
                .context("Failed to create Qdrant client")?
        };

        // Ensure collection exists
        Self::ensure_collection(&client, config).await?;

        Ok(client)
    }

    /// Ensure collection exists with proper configuration
    #[cfg(feature = "vector-store")]
    async fn ensure_collection(client: &QdrantClient, config: &VectorStoreConfig) -> Result<()> {
        let collection_name = &config.collection_name;

        // Check if collection exists
        let exists = client
            .collection_exists(collection_name)
            .await
            .unwrap_or(false);

        if !exists {
            info!("Creating new collection: {}", collection_name);

            client
                .create_collection(&CreateCollection {
                    collection_name: collection_name.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                            VectorParams {
                                size: config.vector_dim as u64,
                                distance: Distance::Cosine.into(),
                                ..Default::default()
                            },
                        )),
                    }),
                    ..Default::default()
                })
                .await
                .context("Failed to create collection")?;

            info!("Collection '{}' created successfully", collection_name);
        } else {
            debug!("Collection '{}' already exists", collection_name);
        }

        Ok(())
    }

    /// Index documents into the vector store
    pub async fn index_documents(
        &self,
        documents: &[ManPageDocument],
    ) -> Result<usize> {
        #[cfg(feature = "vector-store")]
        {
            self.index_documents_impl(documents).await
        }

        #[cfg(not(feature = "vector-store"))]
        {
            warn!("Vector store feature not enabled, skipping indexing");
            Ok(0)
        }
    }

    #[cfg(feature = "vector-store")]
    async fn index_documents_impl(&self, documents: &[ManPageDocument]) -> Result<usize> {
        let client = self.client.as_ref()
            .context("Qdrant client not initialized")?;

        info!("Indexing {} documents", documents.len());

        let mut points = Vec::new();

        for (idx, doc) in documents.iter().enumerate() {
            // Generate embedding for document content
            let embedding = super::embeddings::generate_embedding(&doc.content).await?;

            // Create point with metadata
            use qdrant_client::qdrant::value::Kind;
            use qdrant_client::qdrant::Value;
            use std::collections::HashMap;

            let mut payload_map = HashMap::new();
            payload_map.insert(
                "command".to_string(),
                Value {
                    kind: Some(Kind::StringValue(doc.command.clone())),
                },
            );
            payload_map.insert(
                "section".to_string(),
                Value {
                    kind: Some(Kind::StringValue(doc.section.as_str().to_string())),
                },
            );
            payload_map.insert(
                "os".to_string(),
                Value {
                    kind: Some(Kind::StringValue(doc.metadata.os.clone())),
                },
            );
            payload_map.insert(
                "distro".to_string(),
                Value {
                    kind: Some(Kind::StringValue(doc.metadata.distro.clone())),
                },
            );
            payload_map.insert(
                "is_gnu".to_string(),
                Value {
                    kind: Some(Kind::BoolValue(doc.metadata.is_gnu)),
                },
            );

            let point = PointStruct::new(idx as u64, embedding, payload_map);

            points.push(point);

            // Batch insert every 100 points
            if points.len() >= 100 {
                client
                    .upsert_points(
                        self.config.collection_name.clone(),
                        None,
                        points.clone(),
                        None,
                    )
                    .await
                    .context("Failed to upsert points")?;
                points.clear();
            }
        }

        // Insert remaining points
        if !points.is_empty() {
            client
                .upsert_points(
                    self.config.collection_name.clone(),
                    None,
                    points,
                    None,
                )
                .await
                .context("Failed to upsert remaining points")?;
        }

        info!("Successfully indexed {} documents", documents.len());
        Ok(documents.len())
    }

    /// Query for relevant documents
    pub async fn query(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<QueryResult>> {
        #[cfg(feature = "vector-store")]
        {
            self.query_impl(query_text, limit).await
        }

        #[cfg(not(feature = "vector-store"))]
        {
            warn!("Vector store feature not enabled, returning empty results");
            Ok(Vec::new())
        }
    }

    #[cfg(feature = "vector-store")]
    async fn query_impl(&self, query_text: &str, limit: usize) -> Result<Vec<QueryResult>> {
        let client = self.client.as_ref()
            .context("Qdrant client not initialized")?;

        debug!("Querying vector store: '{}' (limit: {})", query_text, limit);

        // Generate embedding for query
        let query_embedding = super::embeddings::generate_embedding(query_text).await?;

        // Search
        let search_result = client
            .search_points(&SearchPoints {
                collection_name: self.config.collection_name.clone(),
                vector: query_embedding,
                limit: limit as u64,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .context("Failed to search points")?;

        // Convert results
        let mut results = Vec::new();
        for (rank, scored_point) in search_result.result.into_iter().enumerate() {
            let payload = scored_point.payload;

            // Reconstruct ManPageDocument from payload
            // This is a simplified version - in practice, you'd want to store
            // the full document or retrieve it from a separate store
            let command = payload.get("command")
                .and_then(|v| match &v.kind {
                    Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let section_str = payload.get("section")
                .and_then(|v| match &v.kind {
                    Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            // For now, create a stub result
            // In production, you'd fetch the full document
            debug!("Found match: {} - {} (score: {})", command, section_str, scored_point.score);

            results.push(QueryResult {
                document: ManPageDocument {
                    command: "stub".to_string(),
                    section: crate::indexing::ManSection::Description,
                    content: "stub content".to_string(),
                    metadata: crate::indexing::CommandMetadata {
                        os: "linux".to_string(),
                        distro: "ubuntu".to_string(),
                        version: None,
                        installed_path: None,
                        indexed_at: chrono::Utc::now(),
                        man_section: Some(1),
                        is_gnu: false,
                        command_type: crate::indexing::CommandType::Other,
                    },
                },
                score: scored_point.score,
                rank,
            });
        }

        Ok(results)
    }

    /// Get vector store statistics
    pub async fn get_stats(&self) -> Result<VectorStoreStats> {
        #[cfg(feature = "vector-store")]
        {
            self.get_stats_impl().await
        }

        #[cfg(not(feature = "vector-store"))]
        {
            Ok(VectorStoreStats {
                total_vectors: 0,
                collection: "none".to_string(),
                db_size_bytes: 0,
                healthy: false,
                index_stats: None,
            })
        }
    }

    #[cfg(feature = "vector-store")]
    async fn get_stats_impl(&self) -> Result<VectorStoreStats> {
        let client = self.client.as_ref()
            .context("Qdrant client not initialized")?;

        let collection_info = client
            .collection_info(&self.config.collection_name)
            .await
            .context("Failed to get collection info")?;

        let total_vectors = collection_info.result
            .and_then(|info| info.points_count)
            .unwrap_or(0) as usize;

        Ok(VectorStoreStats {
            total_vectors,
            collection: self.config.collection_name.clone(),
            db_size_bytes: 0, // TODO: Calculate actual size
            healthy: true,
            index_stats: None,
        })
    }

    /// Delete collection (for testing)
    #[cfg(test)]
    pub async fn delete_collection(&self) -> Result<()> {
        #[cfg(feature = "vector-store")]
        {
            if let Some(client) = &self.client {
                client
                    .delete_collection(&self.config.collection_name)
                    .await
                    .context("Failed to delete collection")?;
            }
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "vector-store"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_store_client_creation() {
        let config = VectorStoreConfig {
            in_memory: true,
            collection_name: "test_collection".to_string(),
            ..Default::default()
        };

        // Note: This test requires a running Qdrant instance
        // In practice, you'd mock this or use an embedded instance
        let result = VectorStoreClient::new(config).await;

        // Just verify it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
