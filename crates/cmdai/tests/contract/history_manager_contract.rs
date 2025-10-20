//! Contract Test: History Manager Storage and Retrieval
//! 
//! This test validates the production-ready history management system
//! with SQLite persistence, FTS5 search, and performance requirements.
//! 
//! MUST FAIL: These tests expect implementation of production history system
//! that builds on existing T001-T010 foundation.

use cmdai::history::{HistoryManager, CommandHistoryEntry, SearchQuery, SearchFilters, ExecutionMetadata, SafetyMetadata};
use cmdai::models::{RiskLevel, ShellType};
use anyhow::Result;
use tempfile::tempdir;
use chrono::Utc;
use std::time::Duration;

#[tokio::test]
async fn test_production_history_manager_initialization() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    
    // This should initialize with connection pooling and FTS5 support
    let manager = HistoryManager::new(db_path.to_str().unwrap()).await?;
    
    // Verify database schema includes all production tables
    assert!(manager.has_fts_support().await?);
    assert!(manager.has_embedding_support().await?);
    assert!(manager.connection_pool_size() >= 2);
    
    Ok(())
}

#[tokio::test]
async fn test_store_entry_with_rich_metadata() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    let manager = HistoryManager::new(db_path.to_str().unwrap()).await?;
    
    let execution_metadata = ExecutionMetadata {
        exit_code: Some(0),
        execution_time: Some(Duration::from_millis(150)),
        output_size: Some(1024),
        backend_used: "mlx".to_string(),
        generation_time: Duration::from_millis(1200),
        validation_time: Duration::from_millis(25),
    };
    
    let safety_metadata = SafetyMetadata {
        risk_level: RiskLevel::Safe,
        patterns_matched: vec!["file_listing".to_string()],
        user_confirmed: false,
        safety_score: 0.95,
    };
    
    let entry = CommandHistoryEntry::builder()
        .command("ls -la /home/user/documents")
        .explanation("List all files in documents directory with detailed information")
        .user_input("show me all files in my documents")
        .shell_type(ShellType::Bash)
        .working_directory("/home/user")
        .execution_metadata(execution_metadata)
        .safety_metadata(safety_metadata)
        .tags(vec!["file_management".to_string(), "listing".to_string()])
        .build()?;
    
    // Store with timing validation (constitutional: <10ms)
    let start = std::time::Instant::now();
    manager.store_entry(&entry).await?;
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 10, "History write exceeds constitutional requirement");
    
    // Verify retrieval with all metadata
    let retrieved = manager.get_entry(&entry.id).await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    
    assert_eq!(retrieved.command, entry.command);
    assert_eq!(retrieved.explanation, entry.explanation);
    assert!(retrieved.execution_metadata.is_some());
    assert!(retrieved.safety_metadata.is_some());
    assert_eq!(retrieved.tags.len(), 2);
    
    Ok(())
}

#[tokio::test]
async fn test_fts5_full_text_search() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    let manager = HistoryManager::new(db_path.to_str().unwrap()).await?;
    
    // Store test entries for search
    let entries = vec![
        CommandHistoryEntry::builder()
            .command("find . -name '*.rs' -type f")
            .explanation("Find all Rust source files in current directory")
            .user_input("find rust files")
            .tags(vec!["rust".to_string(), "search".to_string()])
            .build()?,
        CommandHistoryEntry::builder()
            .command("grep -r 'async fn' src/")
            .explanation("Search for async functions in source code")
            .user_input("search for async functions")
            .tags(vec!["rust".to_string(), "async".to_string()])
            .build()?,
        CommandHistoryEntry::builder()
            .command("ls -la")
            .explanation("List directory contents")
            .user_input("list files")
            .tags(vec!["listing".to_string()])
            .build()?,
    ];
    
    for entry in &entries {
        manager.store_entry(entry).await?;
    }
    
    // Test FTS5 text search with timing (constitutional: <50ms)
    let start = std::time::Instant::now();
    let query = SearchQuery::new("rust files")
        .with_filters(SearchFilters::new().with_tags(vec!["rust".to_string()]));
    let results = manager.search(&query).await?;
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 50, "Search exceeds constitutional requirement");
    assert!(!results.is_empty());
    assert!(results.len() >= 2); // Should find both rust-related commands
    
    // Verify relevance ranking
    assert!(results[0].similarity_score >= results[1].similarity_score);
    
    Ok(())
}

#[tokio::test]
async fn test_semantic_search_with_embeddings() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    let manager = HistoryManager::new(db_path.to_str().unwrap()).await?;
    
    // Store entries with embedding vectors
    let entry = CommandHistoryEntry::builder()
        .command("docker ps -a")
        .explanation("List all Docker containers including stopped ones")
        .user_input("show me docker containers")
        .embedding_vector(vec![0.1, 0.2, 0.3, 0.4]) // Mock embedding
        .build()?;
    
    manager.store_entry(&entry).await?;
    
    // Test semantic search that should understand "container management"
    let query = SearchQuery::new("container management")
        .with_similarity_threshold(0.7);
    let results = manager.semantic_search(&query).await?;
    
    assert!(!results.is_empty());
    assert!(results[0].similarity_score >= 0.7);
    
    Ok(())
}

#[tokio::test]
async fn test_pagination_and_filtering() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    let manager = HistoryManager::new(db_path.to_str().unwrap()).await?;
    
    // Store 25 entries for pagination testing
    for i in 0..25 {
        let entry = CommandHistoryEntry::builder()
            .command(format!("echo 'test {}'", i))
            .explanation(format!("Test command number {}", i))
            .user_input(format!("test {}", i))
            .build()?;
        manager.store_entry(&entry).await?;
    }
    
    // Test pagination
    let page1 = manager.get_history_paginated(0, 10).await?;
    assert_eq!(page1.entries.len(), 10);
    assert_eq!(page1.total_count, 25);
    assert_eq!(page1.page, 0);
    assert_eq!(page1.page_size, 10);
    
    let page2 = manager.get_history_paginated(1, 10).await?;
    assert_eq!(page2.entries.len(), 10);
    assert_eq!(page2.page, 1);
    
    let page3 = manager.get_history_paginated(2, 10).await?;
    assert_eq!(page3.entries.len(), 5);
    
    Ok(())
}

#[tokio::test]
async fn test_retention_policy_enforcement() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    let manager = HistoryManager::new(db_path.to_str().unwrap()).await?;
    
    // Store old entry
    let old_entry = CommandHistoryEntry::builder()
        .command("old command")
        .explanation("Old test command")
        .timestamp(Utc::now() - chrono::Duration::days(100))
        .build()?;
    
    // Store new entry  
    let new_entry = CommandHistoryEntry::builder()
        .command("new command")
        .explanation("New test command")
        .build()?;
    
    manager.store_entry(&old_entry).await?;
    manager.store_entry(&new_entry).await?;
    
    // Apply retention policy (cleanup entries older than 30 days)
    let retention_policy = cmdai::history::RetentionPolicy {
        max_age_days: Some(30),
        max_entries: None,
        preserve_favorites: true,
        preserve_frequently_used: true,
    };
    
    let deleted_count = manager.apply_retention_policy(&retention_policy).await?;
    assert_eq!(deleted_count, 1);
    
    // Verify old entry removed, new entry remains
    assert!(manager.get_entry(&old_entry.id).await?.is_none());
    assert!(manager.get_entry(&new_entry.id).await?.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_performance_requirements() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    let manager = HistoryManager::new(db_path.to_str().unwrap()).await?;
    
    // Store 1000 entries for performance testing
    for i in 0..1000 {
        let entry = CommandHistoryEntry::builder()
            .command(format!("performance test {}", i))
            .explanation(format!("Performance test entry {}", i))
            .build()?;
        manager.store_entry(&entry).await?;
    }
    
    // Test write performance (constitutional: <10ms)
    let entry = CommandHistoryEntry::builder()
        .command("performance write test")
        .explanation("Testing write performance")
        .build()?;
    
    let start = std::time::Instant::now();
    manager.store_entry(&entry).await?;
    let write_duration = start.elapsed();
    
    assert!(write_duration.as_millis() < 10, 
        "Write performance {} ms exceeds constitutional requirement of 10ms", 
        write_duration.as_millis());
    
    // Test search performance (constitutional: <50ms for 10K entries)
    let start = std::time::Instant::now();
    let query = SearchQuery::new("performance");
    let _results = manager.search(&query).await?;
    let search_duration = start.elapsed();
    
    assert!(search_duration.as_millis() < 50,
        "Search performance {} ms exceeds constitutional requirement of 50ms",
        search_duration.as_millis());
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("history.db");
    let manager = std::sync::Arc::new(HistoryManager::new(db_path.to_str().unwrap()).await?);
    
    // Test concurrent writes and reads
    let mut handles = vec![];
    
    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let entry = CommandHistoryEntry::builder()
                .command(format!("concurrent test {}", i))
                .explanation(format!("Concurrent operation {}", i))
                .build()
                .unwrap();
            
            manager_clone.store_entry(&entry).await.unwrap();
            
            // Immediately search for it
            let query = SearchQuery::new(&format!("concurrent test {}", i));
            let results = manager_clone.search(&query).await.unwrap();
            assert!(!results.is_empty());
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await?;
    }
    
    Ok(())
}