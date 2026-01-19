//! ChromaDB integration tests
//!
//! These tests require a running ChromaDB server.
//! Start the server with:
//!   cd tests && docker-compose up -d
//!
//! Run tests with:
//!   cargo test --features chromadb --test chromadb_integration -- --ignored --nocapture --test-threads=1
//!
//! IMPORTANT: Tests must run serially (--test-threads=1) because they all use the same
//! collection name ("caro_commands") and interfere with each other when run in parallel.
//! See issue #537 for work to enable parallel test execution with unique collection names.

#[cfg(feature = "chromadb")]
mod chromadb_tests {
    use caro::knowledge::backends::chromadb::ChromaDbBackend;
    use caro::knowledge::backends::VectorBackend;
    use std::env;

    /// Get ChromaDB URL from environment or use default
    fn get_chromadb_url() -> String {
        env::var("CHROMADB_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
    }

    /// Get ChromaDB auth token from environment or use default test token
    fn get_chromadb_auth_token() -> String {
        env::var("CHROMADB_AUTH_TOKEN").unwrap_or_else(|_| "test-token".to_string())
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_connection() {
        let url = get_chromadb_url();
        let auth_token = get_chromadb_auth_token();
        let backend = ChromaDbBackend::new(&url, None, Some(&auth_token))
            .await
            .expect("Failed to create ChromaDB backend");

        assert!(
            backend.is_healthy().await,
            "ChromaDB server should be healthy at {}",
            url
        );
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_record_success() {
        let url = get_chromadb_url();
        let auth_token = get_chromadb_auth_token();
        let backend = ChromaDbBackend::new(&url, None, Some(&auth_token))
            .await
            .expect("Failed to create ChromaDB backend");

        // Clear any existing data
        backend.clear().await.expect("Failed to clear collection");

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
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_record_correction() {
        let url = get_chromadb_url();
        let auth_token = get_chromadb_auth_token();
        let backend = ChromaDbBackend::new(&url, None, Some(&auth_token))
            .await
            .expect("Failed to create ChromaDB backend");

        // Clear any existing data
        backend.clear().await.expect("Failed to clear collection");

        // Record a correction
        backend
            .record_correction(
                "remove directory",
                "rm -rf /",
                "rm -rf ./temp",
                Some("Original command is dangerous - targets root directory"),
                None,
            )
            .await
            .expect("Failed to record correction");

        // Verify stats
        let stats = backend.stats().await.expect("Failed to get stats");
        assert_eq!(
            stats.total_entries, 1,
            "Should have exactly 1 entry after recording correction"
        );
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_find_similar() {
        let url = get_chromadb_url();
        let auth_token = get_chromadb_auth_token();
        let backend = ChromaDbBackend::new(&url, None, Some(&auth_token))
            .await
            .expect("Failed to create ChromaDB backend");

        // Clear any existing data
        backend.clear().await.expect("Failed to clear collection");

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
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_clear() {
        let url = get_chromadb_url();
        let auth_token = get_chromadb_auth_token();
        let backend = ChromaDbBackend::new(&url, None, Some(&auth_token))
            .await
            .expect("Failed to create ChromaDB backend");

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

        // Clear the collection
        backend.clear().await.expect("Failed to clear collection");

        // Verify collection is empty
        let stats_after = backend
            .stats()
            .await
            .expect("Failed to get stats after clear");
        assert_eq!(
            stats_after.total_entries, 0,
            "Collection should be empty after clear"
        );
    }

    #[tokio::test]
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_multiple_operations() {
        let url = get_chromadb_url();
        let auth_token = get_chromadb_auth_token();
        let backend = ChromaDbBackend::new(&url, None, Some(&auth_token))
            .await
            .expect("Failed to create ChromaDB backend");

        // Clear to start fresh
        backend.clear().await.expect("Failed to clear collection");

        // Record multiple successes
        for i in 1..=10 {
            backend
                .record_success(
                    &format!("command {}", i),
                    &format!("echo 'test {}'", i),
                    None,
                    None,
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
    #[ignore] // Requires ChromaDB server
    async fn test_chromadb_context_metadata() {
        let url = get_chromadb_url();
        let auth_token = get_chromadb_auth_token();
        let backend = ChromaDbBackend::new(&url, None, Some(&auth_token))
            .await
            .expect("Failed to create ChromaDB backend");

        // Clear to start fresh
        backend.clear().await.expect("Failed to clear collection");

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
}
