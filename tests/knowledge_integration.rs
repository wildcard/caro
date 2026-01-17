//! Knowledge Index Integration Tests
//!
//! Tests for LanceDB-based knowledge index using FastEmbed for embeddings.
//! These tests require model downloads and are marked as `#[ignore]` by default.
//!
//! ## Running Knowledge Tests
//!
//! ```bash
//! # Run all knowledge tests (downloads models on first run)
//! cargo test --test knowledge_integration --features knowledge -- --ignored
//!
//! # Run specific test
//! cargo test --test knowledge_integration --features knowledge test_record_and_search -- --ignored
//! ```
//!
//! ## Feature Flags
//!
//! These tests only compile when the `knowledge` feature is enabled.

#[cfg(feature = "knowledge")]
mod knowledge_tests {
    use caro::knowledge::KnowledgeIndex;
    use tempfile::TempDir;

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_index_create() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();
        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_record_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();

        // Record a success
        index
            .record_success("list all files", "ls -la", Some("rust project"))
            .await
            .unwrap();

        // Search for similar
        let results = index.find_similar("show files", 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].command, "ls -la");
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_record_correction() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();

        index
            .record_correction(
                "show disk usage",
                "ls -lh",
                "du -h -d 1",
                Some("ls shows files not disk usage"),
            )
            .await
            .unwrap();

        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    #[ignore = "requires model download"]
    async fn test_clear() {
        let temp_dir = TempDir::new().unwrap();
        let index = KnowledgeIndex::open(temp_dir.path()).await.unwrap();

        index
            .record_success("test", "echo test", None)
            .await
            .unwrap();

        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);

        index.clear().await.unwrap();

        let stats = index.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    // TODO: Add test_export_import when PR #492 merges (adds export_to_json/import_from_json methods)
}
