//! Impact estimation for command execution
//!
//! This module predicts the impact of commands before execution by analyzing
//! filesystem state and command patterns.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use super::feature_extractor::CommandFeatures;
use super::ml_predictor::{BlastRadius, ImpactEstimate};

/// Detailed impact analysis with filesystem simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedImpact {
    pub base_estimate: ImpactEstimate,
    pub affected_paths: Vec<PathBuf>,
    pub estimated_bytes: u64,
    pub warnings: Vec<String>,
}

/// Impact estimator for command execution
pub struct ImpactEstimator {
    /// Current working directory for relative path resolution
    cwd: PathBuf,
}

impl ImpactEstimator {
    /// Create new impact estimator
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    /// Estimate impact of command execution
    pub async fn estimate(&self, command: &str, features: &CommandFeatures) -> Result<DetailedImpact> {
        // Start with base estimate from features
        let base_estimate = self.base_estimate(features);

        // Try to analyze actual filesystem impact
        let (affected_paths, estimated_bytes) = self.analyze_filesystem_impact(command, features).await?;

        // Generate warnings
        let warnings = self.generate_warnings(command, features, &affected_paths);

        Ok(DetailedImpact {
            base_estimate,
            affected_paths,
            estimated_bytes,
            warnings,
        })
    }

    /// Generate base impact estimate from features
    fn base_estimate(&self, features: &CommandFeatures) -> ImpactEstimate {
        use super::feature_extractor::TargetScope;

        let blast_radius = match features.target_scope {
            TargetScope::SingleFile => BlastRadius::Local,
            TargetScope::LocalFiles => BlastRadius::Local,
            TargetScope::Recursive => BlastRadius::Project,
            TargetScope::System => BlastRadius::System,
            TargetScope::Root => BlastRadius::System,
            TargetScope::Network => BlastRadius::Network,
        };

        let files_affected = if features.has_recursive_flag {
            Some(1000)
        } else if features.has_wildcard {
            Some(100)
        } else {
            Some(1)
        };

        let data_loss_risk = features.destructive_score;

        let is_reversible = !features.is_disk_command &&
                           data_loss_risk < 0.7;

        ImpactEstimate {
            files_affected,
            data_loss_risk,
            is_reversible,
            blast_radius,
            estimated_duration: None,
        }
    }

    /// Analyze actual filesystem to estimate impact
    async fn analyze_filesystem_impact(
        &self,
        command: &str,
        features: &CommandFeatures,
    ) -> Result<(Vec<PathBuf>, u64)> {
        let mut affected_paths = Vec::new();
        let mut estimated_bytes = 0u64;

        // Extract target paths from command
        let targets = self.extract_target_paths(command, features)?;

        for target in targets {
            // Resolve path relative to CWD
            let path = if target.is_absolute() {
                target
            } else {
                self.cwd.join(&target)
            };

            // Skip if path doesn't exist (can't estimate)
            if !path.exists() {
                continue;
            }

            // If recursive operation, walk directory tree
            if features.has_recursive_flag && path.is_dir() {
                let (paths, bytes) = self.walk_directory(&path).await?;
                affected_paths.extend(paths);
                estimated_bytes += bytes;
            } else {
                // Single file or directory
                if let Ok(metadata) = tokio::fs::metadata(&path).await {
                    estimated_bytes += metadata.len();
                    affected_paths.push(path);
                }
            }
        }

        Ok((affected_paths, estimated_bytes))
    }

    /// Extract target paths from command string
    fn extract_target_paths(&self, command: &str, features: &CommandFeatures) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Simple heuristic: look for arguments that look like paths
        for token in &features.tokens {
            // Skip command name and flags
            if token.starts_with('-') || token == &features.tokens[0] {
                continue;
            }

            // Check if token looks like a path
            if token.contains('/') || token.contains('*') || token.contains('.') {
                // Handle wildcards
                if token.contains('*') || token.contains('?') {
                    // Expand glob pattern
                    if let Ok(expanded) = self.expand_glob(token) {
                        paths.extend(expanded);
                    }
                } else {
                    paths.push(PathBuf::from(token));
                }
            }
        }

        // If no paths found, assume current directory
        if paths.is_empty() {
            paths.push(self.cwd.clone());
        }

        Ok(paths)
    }

    /// Expand glob pattern to actual paths
    fn expand_glob(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        use glob::glob;

        let pattern_path = if PathBuf::from(pattern).is_absolute() {
            pattern.to_string()
        } else {
            self.cwd.join(pattern).to_string_lossy().to_string()
        };

        let mut paths = Vec::new();
        for entry in glob(&pattern_path).context("Failed to parse glob pattern")? {
            if let Ok(path) = entry {
                paths.push(path);
            }
        }

        Ok(paths)
    }

    /// Walk directory tree and count files/bytes
    async fn walk_directory(&self, dir: &Path) -> Result<(Vec<PathBuf>, u64)> {
        let mut paths = Vec::new();
        let mut total_bytes = 0u64;

        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            paths.push(path.clone());
            total_bytes += metadata.len();

            // Recursively walk subdirectories
            if metadata.is_dir() {
                let (sub_paths, sub_bytes) = Box::pin(self.walk_directory(&path)).await?;
                paths.extend(sub_paths);
                total_bytes += sub_bytes;
            }
        }

        Ok((paths, total_bytes))
    }

    /// Generate warnings based on impact analysis
    fn generate_warnings(
        &self,
        _command: &str,
        features: &CommandFeatures,
        affected_paths: &[PathBuf],
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // Warn about large number of files
        if affected_paths.len() > 100 {
            warnings.push(format!(
                "This command will affect {} files",
                affected_paths.len()
            ));
        }

        // Warn about system paths
        if affected_paths.iter().any(|p| {
            p.starts_with("/usr") ||
            p.starts_with("/bin") ||
            p.starts_with("/etc") ||
            p.starts_with("/sys")
        }) {
            warnings.push("This command affects system directories".to_string());
        }

        // Warn about recursive operations
        if features.has_recursive_flag && !affected_paths.is_empty() {
            warnings.push("Recursive operation will affect all subdirectories".to_string());
        }

        // Warn about wildcards
        if features.has_wildcard {
            warnings.push("Wildcard pattern may match more files than expected".to_string());
        }

        // Warn about irreversible operations
        if !features.tokens.is_empty() {
            let first = &features.tokens[0];
            if matches!(first.as_str(), "dd" | "mkfs" | "shred") {
                warnings.push("This operation is IRREVERSIBLE".to_string());
            }
        }

        warnings
    }
}

/// Format bytes in human-readable form
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio;

    #[tokio::test]
    async fn test_impact_estimator_basic() {
        let temp_dir = TempDir::new().unwrap();
        let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());

        let features = CommandFeatures::extract("ls -la");
        let impact = estimator.estimate("ls -la", &features).await.unwrap();

        assert_eq!(impact.base_estimate.blast_radius, BlastRadius::Local);
    }

    #[tokio::test]
    async fn test_walk_directory() {
        let temp_dir = TempDir::new().unwrap();
        let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());

        // Create test files
        tokio::fs::write(temp_dir.path().join("file1.txt"), "test").await.unwrap();
        tokio::fs::write(temp_dir.path().join("file2.txt"), "test").await.unwrap();

        let (paths, bytes) = estimator.walk_directory(temp_dir.path()).await.unwrap();

        assert_eq!(paths.len(), 2);
        assert_eq!(bytes, 8); // 2 files * 4 bytes each
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_extract_target_paths() {
        let temp_dir = TempDir::new().unwrap();
        let estimator = ImpactEstimator::new(temp_dir.path().to_path_buf());

        let features = CommandFeatures::extract("rm /tmp/test.txt");
        let paths = estimator.extract_target_paths("rm /tmp/test.txt", &features).unwrap();

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("/tmp/test.txt"));
    }
}
