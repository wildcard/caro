//! Knowledge index integration tests (LanceDB backend)
//!
//! These tests verify the knowledge index functionality using the default
//! LanceDB backend. They don't require external services and use a temporary
//! directory for test data.
//!
//! Run with:
//!   cargo test --features knowledge --test knowledge_integration

#[cfg(feature = "knowledge")]
mod knowledge_tests {
    use caro::knowledge::backends::lancedb::LanceDbBackend;
    use caro::knowledge::backends::VectorBackend;
    use tempfile::TempDir;

    /// Helper to create a temporary LanceDB backend for testing
    async fn create_test_backend() -> (LanceDbBackend, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let backend = LanceDbBackend::new(temp_dir.path())
            .await
            .expect("Failed to create LanceDB backend");
        (backend, temp_dir)
    }

    #[tokio::test]
    async fn test_lancedb_health() {
        let (backend, _temp_dir) = create_test_backend().await;
        assert!(
            backend.is_healthy().await,
            "LanceDB backend should be healthy"
        );
    }

    #[tokio::test]
    async fn test_lancedb_record_success() {
        let (backend, _temp_dir) = create_test_backend().await;

        // Record a success
        backend
            .record_success("list files", "ls -la", Some("/home/user"), None)
            .await
            .expect("Failed to record success");

        // Verify stats
        let stats = backend.stats().await.expect("Failed to get stats");
        assert_eq!(
            stats.total_entries, 1,
            "Should have exactly 1 entry after recording success"
        );
        assert_eq!(stats.success_count, 1, "Should have 1 success entry");
        assert_eq!(
            stats.correction_count, 0,
            "Should have 0 correction entries"
        );
    }

    #[tokio::test]
    async fn test_lancedb_record_correction() {
        let (backend, _temp_dir) = create_test_backend().await;

        // Record a correction
        backend
            .record_correction(
                "remove directory",
                "rm -rf /",
                "rm -rf ./temp",
                Some("Original command is dangerous - targets root directory"),
                None, // profile
            )
            .await
            .expect("Failed to record correction");

        // Verify stats
        let stats = backend.stats().await.expect("Failed to get stats");
        assert_eq!(
            stats.total_entries, 1,
            "Should have exactly 1 entry after recording correction"
        );

        // Note: LanceDB stats() currently counts all entries as successes
        // TODO: Update when type-specific counting is implemented
        // See src/knowledge/backends/lancedb.rs:301-326
    }

    #[tokio::test]
    async fn test_lancedb_find_similar() {
        let (backend, _temp_dir) = create_test_backend().await;

        // Record some commands
        backend
            .record_success("list files", "ls -la", None, None)
            .await
            .expect("Failed to record first command");

        backend
            .record_success("show files", "ls -lh", None, None)
            .await
            .expect("Failed to record second command");

        backend
            .record_success("remove file", "rm file.txt", None, None)
            .await
            .expect("Failed to record third command");

        // Search for similar commands
        let results = backend
            .find_similar("display files", 5)
            .await
            .expect("Failed to search for similar commands");

        assert!(
            !results.is_empty(),
            "Should find at least one similar command"
        );
        assert!(results.len() <= 5, "Should not return more than limit (5)");

        // Verify similarity scores are valid
        for result in &results {
            assert!(
                result.similarity >= 0.0 && result.similarity <= 1.0,
                "Similarity score should be between 0.0 and 1.0, got {}",
                result.similarity
            );
        }

        // Most similar should be ls commands
        let top_result = &results[0];
        assert!(
            top_result.command.starts_with("ls"),
            "Most similar command should be an ls command, got: {}",
            top_result.command
        );
    }

    #[tokio::test]
    async fn test_lancedb_clear() {
        let (backend, _temp_dir) = create_test_backend().await;

        // Add some entries
        backend
            .record_success("test command", "echo test", None, None)
            .await
            .expect("Failed to record command");

        // Verify entry exists
        let stats_before = backend.stats().await.expect("Failed to get stats");
        assert!(
            stats_before.total_entries > 0,
            "Should have entries before clear"
        );

        // Clear the database
        backend.clear().await.expect("Failed to clear database");

        // Verify database is empty
        let stats_after = backend
            .stats()
            .await
            .expect("Failed to get stats after clear");
        assert_eq!(
            stats_after.total_entries, 0,
            "Database should be empty after clear"
        );
    }

    #[tokio::test]
    async fn test_lancedb_multiple_operations() {
        let (backend, _temp_dir) = create_test_backend().await;

        // Record multiple successes
        for i in 1..=10 {
            backend
                .record_success(
                    &format!("command {}", i),
                    &format!("echo 'test {}'", i),
                    None,
                    None, // profile
                )
                .await
                .expect("Failed to record success");
        }

        // Verify all entries were added
        let stats = backend.stats().await.expect("Failed to get stats");
        assert_eq!(
            stats.total_entries, 10,
            "Should have 10 entries after recording 10 commands"
        );

        // Search should work with multiple entries
        let results = backend
            .find_similar("command 5", 3)
            .await
            .expect("Failed to search");
        assert_eq!(results.len(), 3, "Should return exactly 3 results");
    }

    #[tokio::test]
    async fn test_lancedb_context_metadata() {
        let (backend, _temp_dir) = create_test_backend().await;

        // Record command with context
        let context = "/home/user/projects/rust-project";
        backend
            .record_success("build project", "cargo build", Some(context), None)
            .await
            .expect("Failed to record with context");

        // Search and verify context is preserved
        let results = backend
            .find_similar("compile project", 1)
            .await
            .expect("Failed to search");

        assert_eq!(results.len(), 1, "Should find one result");
        assert_eq!(
            results[0].context.as_deref(),
            Some(context),
            "Context should be preserved"
        );
    }

    #[tokio::test]
    async fn test_lancedb_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Create backend and add data
        {
            let backend = LanceDbBackend::new(&path)
                .await
                .expect("Failed to create first backend");

            backend
                .record_success("persistent command", "echo persistent", None, None)
                .await
                .expect("Failed to record command");
        }

        // Create new backend instance using same path
        {
            let backend = LanceDbBackend::new(&path)
                .await
                .expect("Failed to create second backend");

            let stats = backend.stats().await.expect("Failed to get stats");
            assert_eq!(
                stats.total_entries, 1,
                "Data should persist across backend instances"
            );

            let results = backend
                .find_similar("persistent", 1)
                .await
                .expect("Failed to search");
            assert_eq!(results.len(), 1, "Should find persisted command");
            assert_eq!(results[0].command, "echo persistent");
        }
    }
}
