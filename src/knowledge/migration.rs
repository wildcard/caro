//! Knowledge backend migration utilities
//!
//! Provides tools for migrating knowledge between backends (LanceDB ↔ ChromaDB)
//! and exporting/importing knowledge in a portable JSONL format.

use super::{backends::VectorBackend, KnowledgeEntry, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;

/// Version of the export format for schema evolution
const EXPORT_FORMAT_VERSION: u32 = 1;

/// Serializable knowledge entry for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedEntry {
    /// Export format version
    pub version: u32,
    /// Natural language request
    pub request: String,
    /// Shell command
    pub command: String,
    /// Optional context (project/directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Entry type (success or correction)
    pub entry_type: String,
    /// Timestamp when entry was created
    pub timestamp: DateTime<Utc>,
    /// Original command (for corrections)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_command: Option<String>,
    /// Feedback (for corrections)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
    /// User profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

impl From<KnowledgeEntry> for ExportedEntry {
    fn from(entry: KnowledgeEntry) -> Self {
        Self {
            version: EXPORT_FORMAT_VERSION,
            request: entry.request,
            command: entry.command,
            context: entry.context,
            entry_type: entry.entry_type.as_str().to_string(),
            timestamp: entry.timestamp,
            original_command: entry.original_command,
            feedback: entry.feedback,
            profile: entry.profile,
        }
    }
}

/// Export knowledge entries from a backend to a JSONL file
///
/// # Arguments
/// * `backend` - Source backend to export from
/// * `output_path` - Path to write JSONL file
/// * `progress` - Optional progress callback (current, total)
///
/// # Returns
/// Number of entries exported
pub async fn export_entries<P: AsRef<Path>>(
    backend: Arc<dyn VectorBackend>,
    output_path: P,
    progress: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<usize> {
    // Get all entries by querying with empty string (returns all)
    // Note: This is a workaround - ideally we'd have a get_all() method
    let entries = backend.find_similar("", usize::MAX).await?;
    let total = entries.len();

    let file = std::fs::File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    for (i, entry) in entries.iter().enumerate() {
        let exported = ExportedEntry::from(entry.clone());
        let json = serde_json::to_string(&exported)
            .map_err(|e| super::KnowledgeError::Serialization(e.to_string()))?;

        writeln!(writer, "{}", json)?;

        if let Some(ref progress_fn) = progress {
            progress_fn(i + 1, total);
        }
    }

    writer.flush()?;

    Ok(total)
}

/// Import knowledge entries from a JSONL file to a backend
///
/// # Arguments
/// * `backend` - Destination backend to import to
/// * `input_path` - Path to read JSONL file
/// * `progress` - Optional progress callback (current, total)
///
/// # Returns
/// Number of entries imported
pub async fn import_entries<P: AsRef<Path>>(
    backend: Arc<dyn VectorBackend>,
    input_path: P,
    progress: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<usize> {
    let file = std::fs::File::open(input_path)?;
    let reader = BufReader::new(file);

    // First pass: count lines for progress reporting
    let lines: Vec<String> = reader
        .lines()
        .collect::<std::io::Result<Vec<_>>>()?;
    let total = lines.len();

    let mut imported = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let exported: ExportedEntry = serde_json::from_str(line)
            .map_err(|e| super::KnowledgeError::Serialization(format!("Line {}: {}", i + 1, e)))?;

        // Import based on entry type
        match exported.entry_type.as_str() {
            "success" => {
                backend
                    .record_success(
                        &exported.request,
                        &exported.command,
                        exported.context.as_deref(),
                        exported.profile.as_deref(),
                    )
                    .await?;
            }
            "correction" => {
                let original = exported.original_command.unwrap_or_default();
                backend
                    .record_correction(
                        &exported.request,
                        &original,
                        &exported.command,
                        exported.feedback.as_deref(),
                        exported.profile.as_deref(),
                    )
                    .await?;
            }
            _ => {
                return Err(super::KnowledgeError::Serialization(format!(
                    "Unknown entry type: {}",
                    exported.entry_type
                )));
            }
        }

        imported += 1;
        if let Some(ref progress_fn) = progress {
            progress_fn(i + 1, total);
        }
    }

    Ok(imported)
}

/// Migrate knowledge from one backend to another
///
/// # Arguments
/// * `source` - Source backend to migrate from
/// * `destination` - Destination backend to migrate to
/// * `verify` - If true, verify entry counts match after migration
/// * `progress` - Optional progress callback (current, total)
///
/// # Returns
/// Number of entries migrated
pub async fn migrate_backend(
    source: Arc<dyn VectorBackend>,
    destination: Arc<dyn VectorBackend>,
    verify: bool,
    progress: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<usize> {
    // Get all entries from source
    let entries = source.find_similar("", usize::MAX).await?;
    let total = entries.len();

    // Import each entry to destination
    for (i, entry) in entries.iter().enumerate() {
        match entry.entry_type.as_str() {
            "success" => {
                destination
                    .record_success(
                        &entry.request,
                        &entry.command,
                        entry.context.as_deref(),
                        entry.profile.as_deref(),
                    )
                    .await?;
            }
            "correction" => {
                let original = entry.original_command.as_deref().unwrap_or("");
                destination
                    .record_correction(
                        &entry.request,
                        original,
                        &entry.command,
                        entry.feedback.as_deref(),
                        entry.profile.as_deref(),
                    )
                    .await?;
            }
            _ => {}
        }

        if let Some(ref progress_fn) = progress {
            progress_fn(i + 1, total);
        }
    }

    // Verify if requested
    if verify {
        let source_stats = source.stats().await?;
        let dest_stats = destination.stats().await?;

        if source_stats.total_entries != dest_stats.total_entries {
            return Err(super::KnowledgeError::Migration(format!(
                "Entry count mismatch: source has {} entries but destination has {}",
                source_stats.total_entries, dest_stats.total_entries
            )));
        }
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::schema::EntryType;
    use tempfile::TempDir;

    #[test]
    fn test_exported_entry_serialization() {
        let entry = KnowledgeEntry {
            request: "list files".to_string(),
            command: "ls -la".to_string(),
            context: Some("rust project".to_string()),
            similarity: 0.95,
            timestamp: Utc::now(),
            entry_type: EntryType::Success,
            original_command: None,
            feedback: None,
            profile: Some("work".to_string()),
        };

        let exported = ExportedEntry::from(entry);
        let json = serde_json::to_string(&exported).unwrap();
        let deserialized: ExportedEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, EXPORT_FORMAT_VERSION);
        assert_eq!(deserialized.request, "list files");
        assert_eq!(deserialized.command, "ls -la");
        assert_eq!(deserialized.profile, Some("work".to_string()));
    }

    #[tokio::test]
    async fn test_export_import_roundtrip() {
        use crate::knowledge::backends::lancedb::LanceDbBackend;

        let temp_dir = TempDir::new().unwrap();
        let backend: Arc<dyn VectorBackend> = Arc::new(LanceDbBackend::new(temp_dir.path()).await.unwrap());

        // Add some test entries
        backend
            .record_success("test request", "test command", None, Some("test-profile"))
            .await
            .unwrap();

        // Export
        let export_file = temp_dir.path().join("export.jsonl");
        let exported = export_entries(Arc::clone(&backend), &export_file, None)
            .await
            .unwrap();
        assert_eq!(exported, 1);

        // Create new backend for import
        let temp_dir2 = TempDir::new().unwrap();
        let backend2: Arc<dyn VectorBackend> = Arc::new(LanceDbBackend::new(temp_dir2.path()).await.unwrap());

        // Import
        let imported = import_entries(Arc::clone(&backend2), &export_file, None)
            .await
            .unwrap();
        assert_eq!(imported, 1);

        // Verify
        let stats = backend2.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
    }
}
