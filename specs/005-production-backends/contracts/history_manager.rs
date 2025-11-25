// Contract Test: History Manager API
// Tests the command history storage and retrieval system

use cmdai::history::{HistoryManager, CommandHistoryEntry, SearchQuery, SearchFilters};
use chrono::Utc;
use anyhow::Result;

#[tokio::test]
async fn test_store_command_entry() -> Result<()> {
    let manager = HistoryManager::new(":memory:").await?;
    
    let entry = CommandHistoryEntry::builder()
        .command("ls -la")
        .explanation("List all files in directory with details")
        .user_input("show all files")
        .working_directory("/home/user")
        .build()?;
    
    // Store the entry
    manager.store_entry(&entry).await?;
    
    // Verify it can be retrieved
    let retrieved = manager.get_entry(&entry.id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().command, "ls -la");
    
    Ok(())
}

#[tokio::test]
async fn test_search_commands_by_text() -> Result<()> {
    let manager = HistoryManager::new(":memory:").await?;
    
    // Store test entries
    let entry1 = CommandHistoryEntry::builder()
        .command("find . -name '*.rs'")
        .explanation("Find all Rust files")
        .user_input("find rust files")
        .build()?;
    
    let entry2 = CommandHistoryEntry::builder()
        .command("ls -la")
        .explanation("List directory contents")
        .user_input("list files")
        .build()?;
    
    manager.store_entry(&entry1).await?;
    manager.store_entry(&entry2).await?;
    
    // Search for rust-related commands
    let query = SearchQuery::new("rust files");
    let results = manager.search(&query).await?;
    
    assert!(!results.is_empty());
    assert!(results[0].entry.command.contains("*.rs"));
    
    Ok(())
}

#[tokio::test]
async fn test_get_history_with_pagination() -> Result<()> {
    let manager = HistoryManager::new(":memory:").await?;
    
    // Store multiple entries
    for i in 0..25 {
        let entry = CommandHistoryEntry::builder()
            .command(format!("echo 'test {}'", i))
            .explanation(format!("Test command {}", i))
            .build()?;
        manager.store_entry(&entry).await?;
    }
    
    // Get first page
    let page1 = manager.get_history(0, 10).await?;
    assert_eq!(page1.len(), 10);
    
    // Get second page
    let page2 = manager.get_history(10, 10).await?;
    assert_eq!(page2.len(), 10);
    
    // Get last page
    let page3 = manager.get_history(20, 10).await?;
    assert_eq!(page3.len(), 5);
    
    Ok(())
}

#[tokio::test]
async fn test_delete_entry() -> Result<()> {
    let manager = HistoryManager::new(":memory:").await?;
    
    let entry = CommandHistoryEntry::builder()
        .command("rm -rf test")
        .explanation("Remove test directory")
        .build()?;
    
    // Store and verify
    manager.store_entry(&entry).await?;
    assert!(manager.get_entry(&entry.id).await?.is_some());
    
    // Delete and verify removal
    manager.delete_entry(&entry.id).await?;
    assert!(manager.get_entry(&entry.id).await?.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_update_execution_metadata() -> Result<()> {
    let manager = HistoryManager::new(":memory:").await?;
    
    let mut entry = CommandHistoryEntry::builder()
        .command("echo 'test'")
        .explanation("Test echo command")
        .build()?;
    
    manager.store_entry(&entry).await?;
    
    // Update with execution metadata
    use cmdai::history::ExecutionMetadata;
    use std::time::Duration;
    
    let exec_metadata = ExecutionMetadata {
        exit_code: Some(0),
        execution_time: Some(Duration::from_millis(150)),
        output_size: Some(5),
        backend_used: "mlx".to_string(),
        generation_time: Duration::from_millis(1200),
        validation_time: Duration::from_millis(25),
    };
    
    manager.update_execution_metadata(&entry.id, exec_metadata).await?;
    
    // Verify update
    let updated = manager.get_entry(&entry.id).await?.unwrap();
    assert!(updated.execution_metadata.is_some());
    assert_eq!(updated.execution_metadata.unwrap().exit_code, Some(0));
    
    Ok(())
}

#[tokio::test]
async fn test_cleanup_old_entries() -> Result<()> {
    let manager = HistoryManager::new(":memory:").await?;
    
    // Store entries with different timestamps
    let old_entry = CommandHistoryEntry::builder()
        .command("old command")
        .explanation("Old test command")
        .timestamp(Utc::now() - chrono::Duration::days(100))
        .build()?;
    
    let new_entry = CommandHistoryEntry::builder()
        .command("new command")
        .explanation("New test command")
        .build()?;
    
    manager.store_entry(&old_entry).await?;
    manager.store_entry(&new_entry).await?;
    
    // Cleanup entries older than 30 days
    let deleted_count = manager.cleanup_old_entries(30).await?;
    assert_eq!(deleted_count, 1);
    
    // Verify only new entry remains
    assert!(manager.get_entry(&old_entry.id).await?.is_none());
    assert!(manager.get_entry(&new_entry.id).await?.is_some());
    
    Ok(())
}