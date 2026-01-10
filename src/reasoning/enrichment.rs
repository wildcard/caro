//! Context enrichment strategies for gathering additional information
//!
//! This module provides strategies for enriching the command generation context
//! with additional information like directory listings, file trees, and more.

use super::analyzer::{ContextNeed, QueryAnalysis};
use super::config::{ContextFetchPolicy, ReasoningConfig};
use super::project_detection::ProjectContext;
use crate::context::ExecutionContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Errors that can occur during enrichment
#[derive(Debug, Error)]
pub enum EnrichmentError {
    #[error("Failed to execute command: {0}")]
    CommandFailed(String),

    #[error("Permission denied for context fetch: {0}")]
    PermissionDenied(String),

    #[error("Timeout while fetching context")]
    Timeout,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Strategy for enrichment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrichmentStrategy {
    /// Automatically fetch all safe context
    AutoFetch,
    /// Ask permission before running commands
    AskPermission,
    /// Only use existing information
    PassiveOnly,
    /// Skip enrichment entirely
    Skip,
}

/// Enriched context gathered for a query
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnrichedContext {
    /// Directory listing (ls -la output)
    pub directory_listing: Vec<String>,

    /// File tree structure
    pub file_tree: String,

    /// Git status (if in a git repo)
    pub git_status: Option<String>,

    /// Running processes (if needed)
    pub process_list: Option<String>,

    /// Network state (if needed)
    pub network_state: Option<String>,

    /// Environment variables (filtered)
    pub environment_vars: HashMap<String, String>,

    /// Additional context gathered
    pub additional: HashMap<String, String>,

    /// Commands that were run to gather context
    pub commands_run: Vec<String>,

    /// Any errors encountered
    pub errors: Vec<String>,
}

impl EnrichedContext {
    /// Create a new empty enriched context
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any context was gathered
    pub fn has_content(&self) -> bool {
        !self.directory_listing.is_empty()
            || !self.file_tree.is_empty()
            || self.git_status.is_some()
            || self.process_list.is_some()
            || !self.additional.is_empty()
    }

    /// Convert to a prompt-friendly string
    pub fn to_prompt_context(&self) -> String {
        let mut parts = Vec::new();

        if !self.directory_listing.is_empty() {
            parts.push(format!(
                "DIRECTORY CONTENTS:\n{}",
                self.directory_listing.join("\n")
            ));
        }

        if !self.file_tree.is_empty() {
            parts.push(format!("FILE STRUCTURE:\n{}", self.file_tree));
        }

        if let Some(ref status) = self.git_status {
            parts.push(format!("GIT STATUS:\n{}", status));
        }

        if let Some(ref procs) = self.process_list {
            parts.push(format!("RUNNING PROCESSES:\n{}", procs));
        }

        for (key, value) in &self.additional {
            parts.push(format!("{}:\n{}", key.to_uppercase(), value));
        }

        parts.join("\n\n")
    }
}

/// Enriches context by gathering additional information
pub struct ContextEnricher {
    config: ReasoningConfig,
}

impl ContextEnricher {
    /// Create a new context enricher
    pub fn new(config: ReasoningConfig) -> Self {
        Self { config }
    }

    /// Enrich context based on analysis needs
    pub async fn enrich(
        &self,
        analysis: &QueryAnalysis,
        exec_context: &ExecutionContext,
        project_context: Option<&ProjectContext>,
    ) -> Result<EnrichedContext, EnrichmentError> {
        let mut enriched = EnrichedContext::new();

        // Determine what we can fetch based on policy
        let can_auto_fetch = matches!(
            self.config.context_fetch_policy,
            ContextFetchPolicy::AutoFetch | ContextFetchPolicy::SafeOnly
        );

        // Process each context need
        for need in &analysis.context_needs {
            let result = match need {
                ContextNeed::DirectoryListing => {
                    if can_auto_fetch {
                        self.fetch_directory_listing(&exec_context.cwd, &mut enriched)
                    } else {
                        Ok(())
                    }
                }
                ContextNeed::FileTree => {
                    if can_auto_fetch {
                        self.fetch_file_tree(&exec_context.cwd, &mut enriched)
                    } else {
                        Ok(())
                    }
                }
                ContextNeed::GitStatus => {
                    if can_auto_fetch && project_context.map_or(false, |p| p.is_git_repo) {
                        self.fetch_git_status(&exec_context.cwd, &mut enriched)
                    } else {
                        Ok(())
                    }
                }
                ContextNeed::ProcessList => {
                    // Process list requires explicit permission
                    if self.config.context_fetch_policy == ContextFetchPolicy::AutoFetch {
                        self.fetch_process_list(&mut enriched)
                    } else {
                        Ok(())
                    }
                }
                ContextNeed::NetworkState => {
                    // Network state requires explicit permission
                    if self.config.context_fetch_policy == ContextFetchPolicy::AutoFetch {
                        self.fetch_network_state(&mut enriched, &exec_context.os)
                    } else {
                        Ok(())
                    }
                }
                ContextNeed::ProjectType | ContextNeed::PackageManager => {
                    // Already handled by project detection
                    Ok(())
                }
                ContextNeed::AvailableTools => {
                    // Already in execution context
                    Ok(())
                }
                ContextNeed::OsInfo => {
                    // Already in execution context
                    Ok(())
                }
                ContextNeed::EnvironmentVars => {
                    self.fetch_environment_vars(&mut enriched)
                }
            };

            if let Err(e) = result {
                enriched.errors.push(format!("{:?}: {}", need, e));
            }
        }

        Ok(enriched)
    }

    /// Fetch directory listing (ls -la equivalent)
    fn fetch_directory_listing(
        &self,
        path: &Path,
        enriched: &mut EnrichedContext,
    ) -> Result<(), EnrichmentError> {
        let entries = std::fs::read_dir(path)?;

        let mut listing = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();

                let metadata = entry.metadata();
                let (file_type, size) = match metadata {
                    Ok(m) => {
                        let ft = if m.is_dir() {
                            "d"
                        } else if m.is_symlink() {
                            "l"
                        } else {
                            "-"
                        };
                        (ft, m.len())
                    }
                    Err(_) => ("?", 0),
                };

                listing.push(format!("{} {:>10} {}", file_type, size, name));
            }
        }

        listing.sort();
        enriched.directory_listing = listing;
        enriched.commands_run.push("readdir()".to_string());

        Ok(())
    }

    /// Fetch file tree structure
    fn fetch_file_tree(
        &self,
        path: &Path,
        enriched: &mut EnrichedContext,
    ) -> Result<(), EnrichmentError> {
        let mut tree = String::new();
        self.build_tree(path, "", &mut tree, 0, self.config.max_tree_depth)?;
        enriched.file_tree = tree;
        enriched.commands_run.push("tree (internal)".to_string());
        Ok(())
    }

    /// Build tree structure recursively
    fn build_tree(
        &self,
        path: &Path,
        prefix: &str,
        output: &mut String,
        depth: usize,
        max_depth: usize,
    ) -> Result<(), EnrichmentError> {
        if depth >= max_depth {
            return Ok(());
        }

        let entries: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();

        let mut entries: Vec<_> = entries.iter().collect();
        entries.sort_by_key(|e| e.file_name());

        // Skip hidden files and common ignored directories
        let ignored = [".git", "node_modules", "target", "__pycache__", ".venv", "venv"];

        let entries: Vec<_> = entries
            .into_iter()
            .filter(|e| {
                let name = e.file_name();
                let name_str = name.to_string_lossy();
                !name_str.starts_with('.') && !ignored.contains(&name_str.as_ref())
            })
            .collect();

        let count = entries.len();

        for (i, entry) in entries.iter().enumerate() {
            let is_last = i == count - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let name = entry.file_name();
            let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);

            if is_dir {
                output.push_str(&format!("{}{}{}/\n", prefix, connector, name.to_string_lossy()));
                let new_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };
                self.build_tree(&entry.path(), &new_prefix, output, depth + 1, max_depth)?;
            } else {
                output.push_str(&format!("{}{}{}\n", prefix, connector, name.to_string_lossy()));
            }
        }

        Ok(())
    }

    /// Fetch git status
    fn fetch_git_status(
        &self,
        path: &Path,
        enriched: &mut EnrichedContext,
    ) -> Result<(), EnrichmentError> {
        let output = Command::new("git")
            .args(["status", "--short"])
            .current_dir(path)
            .output()
            .map_err(|e| EnrichmentError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).to_string();
            enriched.git_status = Some(if status.is_empty() {
                "Clean working directory".to_string()
            } else {
                status
            });
        }

        enriched.commands_run.push("git status --short".to_string());
        Ok(())
    }

    /// Fetch running processes
    fn fetch_process_list(&self, enriched: &mut EnrichedContext) -> Result<(), EnrichmentError> {
        let output = Command::new("ps")
            .args(["aux"])
            .output()
            .map_err(|e| EnrichmentError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            let procs = String::from_utf8_lossy(&output.stdout);
            // Just get first 20 lines to avoid too much context
            let truncated: String = procs.lines().take(20).collect::<Vec<_>>().join("\n");
            enriched.process_list = Some(truncated);
        }

        enriched.commands_run.push("ps aux".to_string());
        Ok(())
    }

    /// Fetch network state
    fn fetch_network_state(
        &self,
        enriched: &mut EnrichedContext,
        os: &str,
    ) -> Result<(), EnrichmentError> {
        let (cmd, args) = if os == "macos" {
            ("lsof", vec!["-iTCP", "-sTCP:LISTEN", "-n", "-P"])
        } else {
            ("ss", vec!["-tuln"])
        };

        let output = Command::new(cmd)
            .args(&args)
            .output()
            .map_err(|e| EnrichmentError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            let state = String::from_utf8_lossy(&output.stdout);
            // Truncate to first 15 lines
            let truncated: String = state.lines().take(15).collect::<Vec<_>>().join("\n");
            enriched.network_state = Some(truncated);
        }

        enriched
            .commands_run
            .push(format!("{} {}", cmd, args.join(" ")));
        Ok(())
    }

    /// Fetch filtered environment variables
    fn fetch_environment_vars(&self, enriched: &mut EnrichedContext) -> Result<(), EnrichmentError> {
        // Only include safe, relevant environment variables
        let allowed_prefixes = [
            "PATH", "HOME", "USER", "SHELL", "TERM", "LANG", "LC_",
            "NODE_", "CARGO_", "GOPATH", "PYTHONPATH", "VIRTUAL_ENV",
            "EDITOR", "VISUAL", "PAGER",
        ];

        let sensitive_patterns = [
            "KEY", "SECRET", "TOKEN", "PASSWORD", "PASSWD", "AUTH",
            "CREDENTIAL", "PRIVATE", "AWS_", "API_",
        ];

        for (key, value) in std::env::vars() {
            // Skip sensitive variables
            let is_sensitive = sensitive_patterns
                .iter()
                .any(|p| key.to_uppercase().contains(p));

            if is_sensitive {
                continue;
            }

            // Only include allowed variables
            let is_allowed = allowed_prefixes.iter().any(|p| key.starts_with(p));

            if is_allowed {
                // Truncate long values
                let truncated = if value.len() > 100 {
                    format!("{}...", &value[..100])
                } else {
                    value
                };
                enriched.environment_vars.insert(key, truncated);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_enriched_context_default() {
        let ctx = EnrichedContext::default();
        assert!(!ctx.has_content());
    }

    #[test]
    fn test_fetch_directory_listing() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let config = ReasoningConfig::default();
        let enricher = ContextEnricher::new(config);
        let mut enriched = EnrichedContext::new();

        enricher.fetch_directory_listing(temp_dir.path(), &mut enriched).unwrap();

        assert!(!enriched.directory_listing.is_empty());
        assert!(enriched.directory_listing.iter().any(|l| l.contains("test.txt")));
        assert!(enriched.directory_listing.iter().any(|l| l.contains("subdir")));
    }

    #[test]
    fn test_fetch_file_tree() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/main.rs"), "").unwrap();

        let config = ReasoningConfig::default();
        let enricher = ContextEnricher::new(config);
        let mut enriched = EnrichedContext::new();

        enricher.fetch_file_tree(temp_dir.path(), &mut enriched).unwrap();

        assert!(!enriched.file_tree.is_empty());
        assert!(enriched.file_tree.contains("file1.txt"));
        assert!(enriched.file_tree.contains("src/"));
    }

    #[test]
    fn test_environment_vars_filtering() {
        let config = ReasoningConfig::default();
        let enricher = ContextEnricher::new(config);
        let mut enriched = EnrichedContext::new();

        enricher.fetch_environment_vars(&mut enriched).unwrap();

        // Should have some vars
        // But should not have sensitive ones
        for key in enriched.environment_vars.keys() {
            assert!(!key.contains("SECRET"));
            assert!(!key.contains("PASSWORD"));
            assert!(!key.contains("TOKEN"));
        }
    }
}
