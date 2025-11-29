//! ChromaDB client wrapper for knowledge base operations

use super::{KnowledgeBaseConfig, KnowledgeBaseError};
use chromadb::client::{ChromaClient, ChromaClientOptions};
use chromadb::collection::{ChromaCollection, CollectionEntries, GetOptions, GetResult, QueryOptions, QueryResult};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Client wrapper for ChromaDB operations
pub struct KnowledgeBaseClient {
    client: ChromaClient,
    config: KnowledgeBaseConfig,
}

impl KnowledgeBaseClient {
    /// Create a new knowledge base client
    pub async fn new(config: KnowledgeBaseConfig) -> Result<Self, KnowledgeBaseError> {
        config.validate().map_err(|e| KnowledgeBaseError::ConfigError {
            message: e.to_string(),
        })?;

        if !config.enabled {
            return Err(KnowledgeBaseError::ConfigError {
                message: "Knowledge base is disabled in configuration".to_string(),
            });
        }

        debug!("Connecting to ChromaDB at {}", config.chroma_url);

        let options = ChromaClientOptions::default().with_url(&config.chroma_url);
        let client = ChromaClient::new(options);

        // Test connection
        match client.heartbeat().await {
            Ok(_) => {
                info!("Successfully connected to ChromaDB");
                Ok(Self { client, config })
            }
            Err(e) => {
                warn!("Failed to connect to ChromaDB: {}", e);
                Err(KnowledgeBaseError::ConnectionError {
                    message: format!("Failed to connect to ChromaDB: {}", e),
                })
            }
        }
    }

    /// Get or create a collection
    pub async fn get_or_create_collection(
        &self,
        name: &str,
    ) -> Result<ChromaCollection, KnowledgeBaseError> {
        debug!("Getting or creating collection: {}", name);

        self.client
            .get_or_create_collection(name, None)
            .await
            .map_err(|e| KnowledgeBaseError::CollectionError {
                message: format!("Failed to get or create collection '{}': {}", name, e),
            })
    }

    /// Add documents to a collection
    pub async fn add_documents(
        &self,
        collection: &ChromaCollection,
        ids: Vec<String>,
        embeddings: Option<Vec<Vec<f32>>>,
        metadatas: Option<Vec<HashMap<String, JsonValue>>>,
        documents: Option<Vec<String>>,
    ) -> Result<(), KnowledgeBaseError> {
        let entries = CollectionEntries {
            ids,
            embeddings,
            metadatas,
            documents,
        };

        collection
            .add(entries, None)
            .await
            .map_err(|e| KnowledgeBaseError::CollectionError {
                message: format!("Failed to add documents: {}", e),
            })
    }

    /// Query a collection for similar documents
    pub async fn query_collection(
        &self,
        collection: &ChromaCollection,
        query_texts: Vec<String>,
        n_results: Option<usize>,
        where_clause: Option<HashMap<String, JsonValue>>,
    ) -> Result<QueryResult, KnowledgeBaseError> {
        let n = n_results.unwrap_or(self.config.max_retrieval_docs);

        let options = QueryOptions {
            query_texts: Some(query_texts),
            n_results: Some(n),
            where_metadata: where_clause,
            ..Default::default()
        };

        collection
            .query(options)
            .await
            .map_err(|e| KnowledgeBaseError::RetrievalError {
                message: format!("Failed to query collection: {}", e),
            })
    }

    /// Get documents from a collection
    pub async fn get_documents(
        &self,
        collection: &ChromaCollection,
        ids: Option<Vec<String>>,
        where_clause: Option<HashMap<String, JsonValue>>,
        limit: Option<usize>,
    ) -> Result<GetResult, KnowledgeBaseError> {
        let options = GetOptions {
            ids,
            where_metadata: where_clause,
            limit,
            offset: None,
            where_document: None,
            include: None,
        };

        collection.get(options).await.map_err(|e| KnowledgeBaseError::RetrievalError {
            message: format!("Failed to get documents: {}", e),
        })
    }

    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> Result<(), KnowledgeBaseError> {
        debug!("Deleting collection: {}", name);

        self.client
            .delete_collection(name)
            .await
            .map_err(|e| KnowledgeBaseError::CollectionError {
                message: format!("Failed to delete collection '{}': {}", name, e),
            })
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<String>, KnowledgeBaseError> {
        self.client
            .list_collections()
            .await
            .map(|collections| {
                collections
                    .into_iter()
                    .map(|c| c.name)
                    .collect()
            })
            .map_err(|e| KnowledgeBaseError::CollectionError {
                message: format!("Failed to list collections: {}", e),
            })
    }

    /// Get the configuration
    pub fn config(&self) -> &KnowledgeBaseConfig {
        &self.config
    }

    /// Check if ChromaDB is healthy
    pub async fn is_healthy(&self) -> bool {
        self.client.heartbeat().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires ChromaDB server running
    async fn test_client_connection() {
        let config = KnowledgeBaseConfig::default();
        let result = KnowledgeBaseClient::new(config).await;

        match result {
            Ok(client) => {
                assert!(client.is_healthy().await);
            }
            Err(e) => {
                eprintln!("ChromaDB not available (expected in CI): {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server running
    async fn test_collection_operations() {
        let config = KnowledgeBaseConfig::default();
        let client = KnowledgeBaseClient::new(config).await.expect("Failed to create client");

        let collection_name = "test_collection";

        // Create collection
        let collection = client
            .get_or_create_collection(collection_name)
            .await
            .expect("Failed to create collection");

        // List collections
        let collections = client.list_collections().await.expect("Failed to list collections");
        assert!(collections.contains(&collection_name.to_string()));

        // Clean up
        client
            .delete_collection(collection_name)
            .await
            .expect("Failed to delete collection");
    }
}
