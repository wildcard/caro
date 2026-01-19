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
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ChromaDB-based vector backend (server-based)
///
/// Supports multi-collection architecture with separate collections for:
/// - Commands: Successful command executions
/// - Corrections: Agentic loop refinements
/// - Docs: Indexed documentation
/// - Preferences: User-specific patterns
/// - Context: Project-specific knowledge
pub struct ChromaDbBackend {
    client: ChromaClient,
    collections: Arc<RwLock<HashMap<CollectionType, ChromaCollection>>>,
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

        // Initialize empty collections HashMap
        // Collections will be created lazily via ensure_collection()
        let collections = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            client,
            collections,
            embedder,
        })
    }

    /// Ensure collection exists for the given type, creating it if necessary
    ///
    /// # Arguments
    /// * `collection_type` - The type of collection to ensure exists
    async fn ensure_collection(&self, collection_type: CollectionType) -> Result<()> {
        // Fast path: check if collection already exists
        {
            let coll_guard = self.collections.read().await;
            if coll_guard.contains_key(&collection_type) {
                return Ok(());
            }
        }

        // Slow path: create the collection
        let mut coll_guard = self.collections.write().await;

        // Double-check after acquiring write lock (another thread might have created it)
        if coll_guard.contains_key(&collection_type) {
            return Ok(());
        }

        let collection_name = collection_type.name();
        let new_collection = self
            .client
            .get_or_create_collection(collection_name, None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        coll_guard.insert(collection_type, new_collection);
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
        profile: Option<&str>,
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

        if let Some(prof) = profile {
            metadata.insert("profile".to_string(), json!(prof));
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

            let profile = metadata
                .get("profile")
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
                profile,
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
        profile: Option<&str>,
    ) -> Result<()> {
        // Commands go into the Commands collection
        self.ensure_collection(CollectionType::Commands).await?;

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
            profile,
        );

        let entries = CollectionEntries {
            ids: vec![&id],
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata]),
            documents: Some(vec![command]),
        };

        let coll_guard = self.collections.read().await;
        let collection = coll_guard.get(&CollectionType::Commands).ok_or_else(|| {
            KnowledgeError::Database(
                "Commands collection not initialized after ensure_collection".into(),
            )
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
        profile: Option<&str>,
    ) -> Result<()> {
        // Corrections go into the Corrections collection
        self.ensure_collection(CollectionType::Corrections).await?;

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
            profile,
        );

        let entries = CollectionEntries {
            ids: vec![&id],
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata]),
            documents: Some(vec![corrected]),
        };

        let coll_guard = self.collections.read().await;
        let collection = coll_guard
            .get(&CollectionType::Corrections)
            .ok_or_else(|| {
                KnowledgeError::Database(
                    "Corrections collection not initialized after ensure_collection".into(),
                )
            })?;

        collection
            .add(entries, None)
            .await
            .map_err(|e| KnowledgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_similar(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>> {
        // Default find_similar searches across all collections
        self.find_similar_in(query, limit, QueryScope::All).await
    }

    async fn stats(&self) -> Result<BackendStats> {
        let coll_guard = self.collections.read().await;

        let mut total_entries = 0;
        let mut success_count = 0;
        let mut correction_count = 0;

        // Aggregate stats across all collections
        for (collection_type, collection) in coll_guard.iter() {
            // Get count for this collection
            let count = collection.count().await.unwrap_or(0); // Ignore errors for individual collections

            total_entries += count;

            // Count successes and corrections in user-generated collections
            match collection_type {
                CollectionType::Commands => {
                    success_count += count;
                }
                CollectionType::Corrections => {
                    correction_count += count;
                }
                _ => {
                    // Docs, Preferences, Context don't contribute to success/correction counts
                }
            }
        }

        Ok(BackendStats {
            total_entries,
            success_count,
            correction_count,
        })
    }

    async fn clear(&self) -> Result<()> {
        let mut coll_guard = self.collections.write().await;

        // Delete all collections
        let collection_names: Vec<String> =
            coll_guard.keys().map(|ct| ct.name().to_string()).collect();

        for collection_name in collection_names {
            self.client
                .delete_collection(&collection_name)
                .await
                .map_err(|e| KnowledgeError::Database(e.to_string()))?;
        }

        // Clear the HashMap
        coll_guard.clear();
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Check client connectivity
        self.client.heartbeat().await.is_ok()
    }

    async fn add_entry(
        &self,
        entry: KnowledgeEntry,
        collection_type: CollectionType,
    ) -> Result<()> {
        // Ensure the target collection exists
        self.ensure_collection(collection_type).await?;

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

        // Add to ChromaDB collection
        let id = uuid::Uuid::new_v4().to_string();
        let document = entry.command.clone();

        let entries = CollectionEntries {
            ids: vec![id.as_str()],
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata]),
            documents: Some(vec![document.as_str()]),
        };

        // Get the collection and add the entry
        let coll_guard = self.collections.read().await;
        let collection = coll_guard.get(&collection_type).ok_or_else(|| {
            KnowledgeError::Database(format!(
                "Collection {} not initialized after ensure_collection",
                collection_type.name()
            ))
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
        scope: QueryScope,
    ) -> Result<Vec<KnowledgeEntry>> {
        // Get the collections to search based on the scope
        let collections_to_search = scope.collections();

        // Ensure all collections exist
        for collection_type in &collections_to_search {
            self.ensure_collection(*collection_type).await?;
        }

        // Generate query embedding
        let embedding = self.embedder.embed_one(query)?;

        // Query each collection and collect results
        let mut all_results = Vec::new();
        {
            let coll_guard = self.collections.read().await;

            for collection_type in &collections_to_search {
                if let Some(collection) = coll_guard.get(collection_type) {
                    let query_options = QueryOptions {
                        query_embeddings: Some(vec![embedding.clone()]),
                        n_results: Some(limit),
                        query_texts: None,
                        where_metadata: None,
                        where_document: None,
                        include: Some(vec!["documents", "metadatas", "distances"]),
                    };

                    match collection.query(query_options, None).await {
                        Ok(result) => {
                            // Parse results from this collection - ChromaDB returns batched results
                            let ids = result.ids.first().cloned().unwrap_or_default();
                            let docs = result
                                .documents
                                .as_ref()
                                .and_then(|d| d.first().cloned())
                                .unwrap_or_default();
                            let metas = result
                                .metadatas
                                .as_ref()
                                .and_then(|m| m.first())
                                .map(|meta_vec| meta_vec.iter().filter_map(|m| m.clone()).collect())
                                .unwrap_or_default();
                            let dists = result
                                .distances
                                .as_ref()
                                .and_then(|d| d.first().cloned())
                                .unwrap_or_else(|| vec![0.0; ids.len()]);

                            if !ids.is_empty() {
                                if let Ok(entries) = Self::parse_results(ids, docs, metas, dists) {
                                    all_results.extend(entries);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to query collection {}: {}",
                                collection_type.name(),
                                e
                            );
                        }
                    }
                }
            }
        }

        // Sort all results by similarity score (descending)
        all_results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit total results
        all_results.truncate(limit);

        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper to create backend
    async fn create_test_backend(_test_name: &str) -> ChromaDbBackend {
        let embedder = Embedder::new(None).expect("Failed to create embedder");
        let client = ChromaClient::new(ChromaClientOptions {
            url: Some("http://localhost:8000".to_string()),
            auth: ChromaAuthMethod::None,
            ..Default::default()
        })
        .await
        .expect("Failed to create ChromaDB client");

        // Initialize with empty collections HashMap
        let collections = Arc::new(RwLock::new(HashMap::new()));

        ChromaDbBackend {
            client,
            collections,
            embedder,
        }
    }

    // Unit tests (no external dependencies)

    #[test]
    fn test_build_metadata_success() {
        let timestamp = Utc::now();
        let metadata = ChromaDbBackend::build_metadata(
            EntryType::Success,
            "list files",
            Some("/home/user"),
            timestamp,
            None,
            None,
            None,
        );

        assert_eq!(metadata["entry_type"], "success");
        assert_eq!(metadata["request"], "list files");
        assert_eq!(metadata["context"], "/home/user");
        assert_eq!(metadata["timestamp"], timestamp.timestamp());
        assert!(metadata.get("original_command").is_none());
        assert!(metadata.get("feedback").is_none());
    }

    #[test]
    fn test_build_metadata_correction() {
        let timestamp = Utc::now();
        let metadata = ChromaDbBackend::build_metadata(
            EntryType::Correction,
            "find files",
            None,
            timestamp,
            Some("find -name '*.txt'"),
            Some("Missing current directory"),
            None,
        );

        assert_eq!(metadata["entry_type"], "correction");
        assert_eq!(metadata["request"], "find files");
        assert!(metadata.get("context").is_none());
        assert_eq!(metadata["original_command"], "find -name '*.txt'");
        assert_eq!(metadata["feedback"], "Missing current directory");
        assert_eq!(metadata["timestamp"], timestamp.timestamp());
    }

    #[test]
    fn test_build_metadata_minimal() {
        let timestamp = Utc::now();
        let metadata = ChromaDbBackend::build_metadata(
            EntryType::Success,
            "simple command",
            None,
            timestamp,
            None,
            None,
            None,
        );

        assert_eq!(metadata["entry_type"], "success");
        assert_eq!(metadata["request"], "simple command");
        assert_eq!(metadata["timestamp"], timestamp.timestamp());
        assert!(metadata.get("context").is_none());
        assert!(metadata.get("original_command").is_none());
        assert!(metadata.get("feedback").is_none());
        // Verify only required fields are present
        assert_eq!(metadata.len(), 3); // entry_type, request, timestamp
    }

    #[test]
    fn test_entry_type_as_str() {
        assert_eq!(EntryType::Success.as_str(), "success");
        assert_eq!(EntryType::Correction.as_str(), "correction");
    }

    #[test]
    fn test_metadata_different_entry_types() {
        let timestamp = Utc::now();
        let meta_success = ChromaDbBackend::build_metadata(
            EntryType::Success,
            "test request",
            None,
            timestamp,
            None,
            None,
            None,
        );
        let meta_correction = ChromaDbBackend::build_metadata(
            EntryType::Correction,
            "test request",
            None,
            timestamp,
            None,
            None,
            None,
        );

        assert_ne!(meta_success["entry_type"], meta_correction["entry_type"]);
        assert_eq!(meta_success["request"], meta_correction["request"]);
    }

    #[test]
    fn test_collection_names() {
        // Verify all collection names follow naming convention
        assert_eq!(CollectionType::Commands.name(), "caro_commands");
        assert_eq!(CollectionType::Corrections.name(), "caro_corrections");
        assert_eq!(CollectionType::Docs.name(), "caro_command_docs");
        assert_eq!(CollectionType::Preferences.name(), "caro_user_preferences");
        assert_eq!(CollectionType::Context.name(), "caro_project_context");

        // All should start with "caro_"
        for ct in CollectionType::all() {
            assert!(ct.name().starts_with("caro_"));
        }
    }

    // Integration tests (require ChromaDB server)

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_health() {
        let backend = create_test_backend("health").await;
        assert!(backend.is_healthy().await, "ChromaDB should be healthy");

        // Cleanup: delete test collection
        backend.clear().await.ok();
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_record_and_search() {
        let backend = create_test_backend("record_search").await;

        // Record a success
        backend
            .record_success("list files", "ls -la", Some("/home/user"), None)
            .await
            .expect("Failed to record success");

        // Search for similar
        let results = backend
            .find_similar("show files", 5)
            .await
            .expect("Failed to search");

        assert!(!results.is_empty(), "Should find similar entries");
        assert_eq!(results[0].command, "ls -la");

        // Cleanup: delete test collection
        backend.clear().await.ok();
    }
}
