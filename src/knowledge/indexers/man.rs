//! Man page indexer
//!
//! Indexes Unix/Linux man pages into the Docs collection.
//! Parses man page content and creates knowledge entries for command documentation.

use super::{IndexStats, Indexer, ProgressCallback};
use crate::knowledge::{
    backends::VectorBackend, collections::CollectionType, index::KnowledgeEntry, schema::EntryType,
    Result,
};
use async_trait::async_trait;
use chrono::Utc;
use std::process::Command;
use std::sync::Arc;

/// Man page indexer
///
/// Discovers and indexes system man pages into the knowledge base.
/// Supports filtering by section and command name.
pub struct ManPageIndexer {
    /// Man page sections to index (1-8, or empty for all)
    pub sections: Vec<u8>,
}

impl ManPageIndexer {
    /// Create a new man page indexer
    ///
    /// # Arguments
    /// * `sections` - Man page sections to index (e.g., vec![1, 8] for user commands and admin)
    ///                If empty, indexes all sections.
    pub fn new(sections: Vec<u8>) -> Self {
        Self { sections }
    }

    /// Create indexer for all sections
    pub fn all_sections() -> Self {
        Self::new(vec![])
    }

    /// Create indexer for user commands only (section 1)
    pub fn user_commands() -> Self {
        Self::new(vec![1])
    }

    /// List all available man pages
    fn list_man_pages(&self) -> Result<Vec<(String, String)>> {
        let output = Command::new("man")
            .arg("-k")
            .arg(".")
            .output()
            .map_err(|e| {
                crate::knowledge::KnowledgeError::Indexing(format!(
                    "Failed to list man pages: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(crate::knowledge::KnowledgeError::Indexing(
                "man -k . command failed".to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut pages = Vec::new();

        for line in stdout.lines() {
            // Parse lines like: "ls (1) - list directory contents"
            if let Some((name_section, description)) = line.split_once(" - ") {
                // Extract command name and section
                if let Some(name_part) = name_section.split_whitespace().next() {
                    // Check if there's a section number in parentheses
                    if let Some(section_start) = name_section.find('(') {
                        if let Some(section_end) = name_section.find(')') {
                            let section = &name_section[section_start + 1..section_end];

                            // Filter by section if specified
                            if !self.sections.is_empty() {
                                if let Ok(section_num) = section.parse::<u8>() {
                                    if !self.sections.contains(&section_num) {
                                        continue;
                                    }
                                }
                            }

                            pages.push((
                                format!("{}({})", name_part, section),
                                description.to_string(),
                            ));
                        }
                    }
                }
            }
        }

        Ok(pages)
    }

    /// Get the raw content of a man page
    fn get_man_page_content(&self, name_with_section: &str) -> Result<String> {
        let output = Command::new("man")
            .arg("-P")
            .arg("cat")
            .arg(name_with_section)
            .output()
            .map_err(|e| {
                crate::knowledge::KnowledgeError::Indexing(format!(
                    "Failed to read man page {}: {}",
                    name_with_section, e
                ))
            })?;

        if !output.status.success() {
            return Err(crate::knowledge::KnowledgeError::Indexing(format!(
                "Failed to read man page {}",
                name_with_section
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Extract synopsis and description from man page content
    fn extract_key_content(&self, content: &str, description: &str) -> String {
        let mut result = String::new();
        result.push_str(description);
        result.push_str("\n\n");

        // Try to find SYNOPSIS section
        if let Some(synopsis_start) = content.find("SYNOPSIS") {
            let after_synopsis = &content[synopsis_start + 8..];
            // Find next section (usually all caps)
            if let Some(next_section) = after_synopsis.find("\nDESCRIPTION") {
                let synopsis = after_synopsis[..next_section].trim();
                if !synopsis.is_empty() {
                    result.push_str("Synopsis:\n");
                    result.push_str(synopsis);
                    result.push_str("\n\n");
                }
            }
        }

        // Try to find DESCRIPTION section (first paragraph only)
        if let Some(desc_start) = content.find("DESCRIPTION") {
            let after_desc = &content[desc_start + 11..];
            // Get first 500 chars or until next section
            let desc_chunk: String = after_desc.chars().take(500).collect();

            if let Some(newline_pos) = desc_chunk.find("\n\n") {
                result.push_str("Description:\n");
                result.push_str(desc_chunk[..newline_pos].trim());
            }
        }

        // Limit total length to avoid huge entries
        if result.len() > 1000 {
            result.truncate(1000);
            result.push_str("...");
        }

        result
    }
}

#[async_trait]
impl Indexer for ManPageIndexer {
    fn name(&self) -> &'static str {
        "man"
    }

    async fn index_all(
        &self,
        backend: Arc<dyn VectorBackend>,
        progress: Option<ProgressCallback>,
    ) -> Result<IndexStats> {
        let mut stats = IndexStats::new();

        // List all man pages
        let pages = self.list_man_pages()?;
        let total = pages.len();

        if total == 0 {
            return Ok(stats);
        }

        // Index each page
        for (idx, (name_with_section, short_description)) in pages.iter().enumerate() {
            // Report progress
            if let Some(ref callback) = progress {
                callback(idx, total);
            }

            // Skip if filtering says no
            if !self.should_index(name_with_section) {
                stats.record_skip();
                continue;
            }

            // Try to get and index the man page
            match self.get_man_page_content(name_with_section) {
                Ok(content) => {
                    let documentation = self.extract_key_content(&content, short_description);

                    // Create knowledge entry
                    let entry = KnowledgeEntry {
                        request: name_with_section.clone(), // Command name with section
                        command: documentation,             // Extracted docs
                        context: Some(format!("man-page:{}", name_with_section)),
                        similarity: 0.0, // Not used for indexing
                        timestamp: Utc::now(),
                        entry_type: EntryType::Success, // Documentation as "success" entries
                        original_command: None,
                        feedback: None,
                        profile: None, // Docs are profile-agnostic
                    };

                    // Add to Docs collection
                    match backend.add_entry(entry, CollectionType::Docs).await {
                        Ok(()) => stats.record_success(),
                        Err(_) => stats.record_failure(),
                    }
                }
                Err(_) => {
                    stats.record_failure();
                }
            }
        }

        // Final progress callback
        if let Some(ref callback) = progress {
            callback(total, total);
        }

        Ok(stats)
    }

    async fn index_one(&self, backend: Arc<dyn VectorBackend>, item: &str) -> Result<bool> {
        // Check if man page exists
        let check_output = Command::new("man")
            .arg("-w")
            .arg(item)
            .output()
            .map_err(|e| {
                crate::knowledge::KnowledgeError::Indexing(format!(
                    "Failed to check if man page exists: {}",
                    e
                ))
            })?;

        if !check_output.status.success() {
            // Man page doesn't exist
            return Ok(false);
        }

        // Get short description from apropos
        let apropos_output = Command::new("apropos").arg(item).output().ok();

        let short_description = if let Some(output) = apropos_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse first line like: "ls (1) - list directory contents"
            stdout
                .lines()
                .next()
                .and_then(|line| line.split_once(" - "))
                .map(|(_, desc)| desc.to_string())
                .unwrap_or_else(|| format!("Manual page for {}", item))
        } else {
            format!("Manual page for {}", item)
        };

        // Get full content
        let content = self.get_man_page_content(item)?;
        let documentation = self.extract_key_content(&content, &short_description);

        // Create knowledge entry
        let entry = KnowledgeEntry {
            request: item.to_string(),
            command: documentation,
            context: Some(format!("man-page:{}", item)),
            similarity: 0.0,
            timestamp: Utc::now(),
            entry_type: EntryType::Success,
            original_command: None,
            feedback: None,
            profile: None,
        };

        // Add to Docs collection
        backend.add_entry(entry, CollectionType::Docs).await?;
        Ok(true)
    }

    fn should_index(&self, item: &str) -> bool {
        // Skip internal/deprecated commands
        let skip_list = [
            "builtin",   // Shell builtins
            "intro",     // Introduction pages
            "deprecate", // Deprecated commands
        ];

        let item_lower = item.to_lowercase();
        for skip in &skip_list {
            if item_lower.contains(skip) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_man_page_indexer_new() {
        let indexer = ManPageIndexer::new(vec![1, 8]);
        assert_eq!(indexer.sections, vec![1, 8]);
        assert_eq!(indexer.name(), "man");
    }

    #[test]
    fn test_man_page_indexer_user_commands() {
        let indexer = ManPageIndexer::user_commands();
        assert_eq!(indexer.sections, vec![1]);
    }

    #[test]
    fn test_man_page_indexer_all_sections() {
        let indexer = ManPageIndexer::all_sections();
        assert!(indexer.sections.is_empty());
    }
}
