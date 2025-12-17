//! Batch indexer for creating distribution-specific command indexes

use super::{ManPageDocument, IndexStatistics};
use super::command_scanner::scan_installed_commands;
use super::man_parser::parse_man_page;
use anyhow::{Context, Result};
use std::collections::HashMap;
use chrono::Utc;
use tracing::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle};

/// Batch index all commands on the system
pub async fn batch_index_system(
    os: &str,
    distro: &str,
    max_commands: Option<usize>,
) -> Result<Vec<ManPageDocument>> {
    info!("Starting batch indexing for {} {}", os, distro);

    // Scan for installed commands
    let all_commands = scan_installed_commands()
        .context("Failed to scan installed commands")?;

    // Limit number of commands if specified
    let commands: Vec<String> = if let Some(max) = max_commands {
        all_commands.into_iter().take(max).collect()
    } else {
        all_commands
    };

    info!("Found {} commands to index", commands.len());

    // Create progress bar
    let pb = ProgressBar::new(commands.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut all_documents = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for command in &commands {
        pb.set_message(format!("Indexing {}", command));

        match parse_man_page(command, os, distro) {
            Ok(documents) => {
                all_documents.extend(documents);
                successful += 1;
            }
            Err(e) => {
                warn!("Failed to index command '{}': {}", command, e);
                failed += 1;
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message(format!(
        "Indexed {} commands ({} successful, {} failed)",
        commands.len(),
        successful,
        failed
    ));

    info!(
        "Batch indexing complete: {} documents from {} commands",
        all_documents.len(),
        successful
    );

    Ok(all_documents)
}

/// Generate index statistics
pub fn generate_statistics(
    documents: &[ManPageDocument],
    os: &str,
    distro: &str,
) -> IndexStatistics {
    use super::CommandType;

    let unique_commands: std::collections::HashSet<_> =
        documents.iter().map(|d| &d.command).collect();

    let mut by_type: HashMap<CommandType, usize> = HashMap::new();

    for doc in documents {
        *by_type.entry(doc.metadata.command_type).or_insert(0) += 1;
    }

    // Estimate index size (rough approximation)
    let index_size_bytes: u64 = documents
        .iter()
        .map(|d| {
            (d.command.len() + d.content.len() + d.section.as_str().len()) as u64
        })
        .sum();

    IndexStatistics {
        total_commands: unique_commands.len(),
        total_documents: documents.len(),
        by_type,
        index_size_bytes,
        last_updated: Utc::now(),
        os: os.to_string(),
        distro: distro.to_string(),
    }
}

/// Filter documents to prioritize important sections
pub fn filter_important_sections(documents: Vec<ManPageDocument>) -> Vec<ManPageDocument> {
    use super::ManSection;

    documents
        .into_iter()
        .filter(|doc| {
            matches!(
                doc.section,
                ManSection::Name
                    | ManSection::Synopsis
                    | ManSection::Description
                    | ManSection::Options
                    | ManSection::Examples
            )
        })
        .collect()
}

/// Sample commands for testing (subset of most common)
pub fn get_sample_commands() -> Vec<String> {
    vec![
        "ls", "cat", "grep", "find", "sed", "awk", "ps", "df", "du", "tar",
        "curl", "wget", "git", "ssh", "rsync", "chmod", "chown", "kill",
        "sort", "uniq", "head", "tail", "wc", "cut", "tr", "xargs",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_index_sample() {
        // Test with a small sample to avoid long test times
        let sample_commands = vec!["ls".to_string(), "cat".to_string()];

        let mut all_documents = Vec::new();

        for command in sample_commands {
            if let Ok(documents) = parse_man_page(&command, "linux", "test") {
                all_documents.extend(documents);
            }
        }

        assert!(!all_documents.is_empty());
    }

    #[test]
    fn test_generate_statistics() {
        use super::super::{CommandMetadata, CommandType, ManSection};
        use chrono::Utc;

        let metadata = CommandMetadata {
            os: "linux".to_string(),
            distro: "ubuntu-22.04".to_string(),
            version: Some("1.0".to_string()),
            installed_path: None,
            indexed_at: Utc::now(),
            man_section: Some(1),
            is_gnu: true,
            command_type: CommandType::CoreUtil,
        };

        let documents = vec![
            ManPageDocument {
                command: "ls".to_string(),
                section: ManSection::Name,
                content: "list directory contents".to_string(),
                metadata: metadata.clone(),
            },
            ManPageDocument {
                command: "ls".to_string(),
                section: ManSection::Description,
                content: "List files and directories".to_string(),
                metadata: metadata.clone(),
            },
        ];

        let stats = generate_statistics(&documents, "linux", "ubuntu-22.04");

        assert_eq!(stats.total_commands, 1); // Only 'ls'
        assert_eq!(stats.total_documents, 2);
        assert_eq!(stats.os, "linux");
        assert_eq!(stats.distro, "ubuntu-22.04");
        assert!(stats.index_size_bytes > 0);
    }

    #[test]
    fn test_filter_important_sections() {
        use super::super::{CommandMetadata, CommandType, ManSection};
        use chrono::Utc;

        let metadata = CommandMetadata {
            os: "linux".to_string(),
            distro: "ubuntu-22.04".to_string(),
            version: Some("1.0".to_string()),
            installed_path: None,
            indexed_at: Utc::now(),
            man_section: Some(1),
            is_gnu: true,
            command_type: CommandType::CoreUtil,
        };

        let documents = vec![
            ManPageDocument {
                command: "ls".to_string(),
                section: ManSection::Name,
                content: "list".to_string(),
                metadata: metadata.clone(),
            },
            ManPageDocument {
                command: "ls".to_string(),
                section: ManSection::Author,
                content: "Written by someone".to_string(),
                metadata: metadata.clone(),
            },
            ManPageDocument {
                command: "ls".to_string(),
                section: ManSection::Description,
                content: "Lists files".to_string(),
                metadata: metadata.clone(),
            },
        ];

        let filtered = filter_important_sections(documents);

        // Should keep NAME and DESCRIPTION, filter out AUTHOR
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|d| d.section == ManSection::Name));
        assert!(filtered.iter().any(|d| d.section == ManSection::Description));
        assert!(!filtered.iter().any(|d| d.section == ManSection::Author));
    }
}
