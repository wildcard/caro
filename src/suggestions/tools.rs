//! PATH and installed tools analysis

use super::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

/// A detected tool on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTool {
    /// Name of the tool
    pub name: String,

    /// Path to the executable
    pub path: PathBuf,

    /// Category of the tool
    pub category: ToolCategory,

    /// Version string if detected
    pub version: Option<String>,
}

impl DetectedTool {
    /// Create a new detected tool
    pub fn new(name: impl Into<String>, path: PathBuf, category: ToolCategory) -> Self {
        Self {
            name: name.into(),
            path,
            category,
            version: None,
        }
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Category of tools for suggestion grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    /// Version control (git, hg, svn)
    VersionControl,

    /// Container runtime (docker, podman)
    ContainerRuntime,

    /// Package manager (npm, cargo, pip, brew)
    PackageManager,

    /// Text editor (vim, nvim, emacs, code)
    Editor,

    /// Programming language (python, node, rustc, go)
    Language,

    /// System utility (find, grep, awk, sed, jq)
    SystemUtil,

    /// Network tool (curl, wget, nc, ssh)
    NetworkTool,

    /// Database client (psql, mysql, redis-cli)
    DatabaseClient,

    /// Cloud CLI (aws, gcloud, kubectl, az)
    CloudCli,

    /// Other/uncategorized
    Other,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionControl => write!(f, "Version Control"),
            Self::ContainerRuntime => write!(f, "Containers"),
            Self::PackageManager => write!(f, "Package Manager"),
            Self::Editor => write!(f, "Editor"),
            Self::Language => write!(f, "Language"),
            Self::SystemUtil => write!(f, "System Utility"),
            Self::NetworkTool => write!(f, "Network"),
            Self::DatabaseClient => write!(f, "Database"),
            Self::CloudCli => write!(f, "Cloud"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Tool definitions for detection
struct ToolDefinition {
    name: &'static str,
    category: ToolCategory,
    version_arg: Option<&'static str>,
}

const TOOLS_TO_DETECT: &[ToolDefinition] = &[
    // Version Control
    ToolDefinition { name: "git", category: ToolCategory::VersionControl, version_arg: Some("--version") },
    ToolDefinition { name: "hg", category: ToolCategory::VersionControl, version_arg: Some("--version") },
    ToolDefinition { name: "svn", category: ToolCategory::VersionControl, version_arg: Some("--version") },

    // Container Runtime
    ToolDefinition { name: "docker", category: ToolCategory::ContainerRuntime, version_arg: Some("--version") },
    ToolDefinition { name: "podman", category: ToolCategory::ContainerRuntime, version_arg: Some("--version") },
    ToolDefinition { name: "docker-compose", category: ToolCategory::ContainerRuntime, version_arg: Some("--version") },

    // Package Managers
    ToolDefinition { name: "cargo", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "npm", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "yarn", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "pnpm", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "pip", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "pip3", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "brew", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "apt", category: ToolCategory::PackageManager, version_arg: Some("--version") },
    ToolDefinition { name: "go", category: ToolCategory::PackageManager, version_arg: Some("version") },

    // Editors
    ToolDefinition { name: "vim", category: ToolCategory::Editor, version_arg: Some("--version") },
    ToolDefinition { name: "nvim", category: ToolCategory::Editor, version_arg: Some("--version") },
    ToolDefinition { name: "emacs", category: ToolCategory::Editor, version_arg: Some("--version") },
    ToolDefinition { name: "code", category: ToolCategory::Editor, version_arg: Some("--version") },
    ToolDefinition { name: "nano", category: ToolCategory::Editor, version_arg: Some("--version") },

    // Languages
    ToolDefinition { name: "python", category: ToolCategory::Language, version_arg: Some("--version") },
    ToolDefinition { name: "python3", category: ToolCategory::Language, version_arg: Some("--version") },
    ToolDefinition { name: "node", category: ToolCategory::Language, version_arg: Some("--version") },
    ToolDefinition { name: "rustc", category: ToolCategory::Language, version_arg: Some("--version") },
    ToolDefinition { name: "ruby", category: ToolCategory::Language, version_arg: Some("--version") },
    ToolDefinition { name: "java", category: ToolCategory::Language, version_arg: Some("-version") },
    ToolDefinition { name: "deno", category: ToolCategory::Language, version_arg: Some("--version") },
    ToolDefinition { name: "bun", category: ToolCategory::Language, version_arg: Some("--version") },

    // System Utilities
    ToolDefinition { name: "jq", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "yq", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "rg", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "fd", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "fzf", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "bat", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "exa", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "eza", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "htop", category: ToolCategory::SystemUtil, version_arg: Some("--version") },
    ToolDefinition { name: "tmux", category: ToolCategory::SystemUtil, version_arg: Some("-V") },

    // Network Tools
    ToolDefinition { name: "curl", category: ToolCategory::NetworkTool, version_arg: Some("--version") },
    ToolDefinition { name: "wget", category: ToolCategory::NetworkTool, version_arg: Some("--version") },
    ToolDefinition { name: "ssh", category: ToolCategory::NetworkTool, version_arg: Some("-V") },
    ToolDefinition { name: "nc", category: ToolCategory::NetworkTool, version_arg: None },
    ToolDefinition { name: "httpie", category: ToolCategory::NetworkTool, version_arg: Some("--version") },

    // Database Clients
    ToolDefinition { name: "psql", category: ToolCategory::DatabaseClient, version_arg: Some("--version") },
    ToolDefinition { name: "mysql", category: ToolCategory::DatabaseClient, version_arg: Some("--version") },
    ToolDefinition { name: "redis-cli", category: ToolCategory::DatabaseClient, version_arg: Some("--version") },
    ToolDefinition { name: "mongosh", category: ToolCategory::DatabaseClient, version_arg: Some("--version") },

    // Cloud CLI
    ToolDefinition { name: "aws", category: ToolCategory::CloudCli, version_arg: Some("--version") },
    ToolDefinition { name: "gcloud", category: ToolCategory::CloudCli, version_arg: Some("--version") },
    ToolDefinition { name: "az", category: ToolCategory::CloudCli, version_arg: Some("--version") },
    ToolDefinition { name: "kubectl", category: ToolCategory::CloudCli, version_arg: Some("version --client") },
    ToolDefinition { name: "terraform", category: ToolCategory::CloudCli, version_arg: Some("--version") },
    ToolDefinition { name: "helm", category: ToolCategory::CloudCli, version_arg: Some("version") },
];

/// Analyzes PATH and detects installed tools
pub struct ToolsAnalyzer {
    /// Skip version detection for faster analysis
    skip_versions: bool,
}

impl Default for ToolsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolsAnalyzer {
    /// Create a new tools analyzer
    pub fn new() -> Self {
        Self { skip_versions: false }
    }

    /// Skip version detection for faster analysis
    pub fn skip_versions(mut self) -> Self {
        self.skip_versions = true;
        self
    }

    /// Analyze installed tools
    pub async fn analyze(&self) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();

        for def in TOOLS_TO_DETECT {
            if let Some(tool) = self.detect_tool(def).await {
                tools.push(tool);
            }
        }

        Ok(tools)
    }

    /// Detect a single tool
    async fn detect_tool(&self, def: &ToolDefinition) -> Option<DetectedTool> {
        // Check if tool exists using `which`
        let output = Command::new("which")
            .arg(def.name)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let path_str = String::from_utf8_lossy(&output.stdout);
        let path = PathBuf::from(path_str.trim());

        let mut tool = DetectedTool::new(def.name, path, def.category);

        // Get version if requested and available
        if !self.skip_versions {
            if let Some(version_arg) = def.version_arg {
                if let Some(version) = self.get_version(def.name, version_arg) {
                    tool = tool.with_version(version);
                }
            }
        }

        Some(tool)
    }

    /// Get version string for a tool
    fn get_version(&self, name: &str, version_arg: &str) -> Option<String> {
        let args: Vec<&str> = version_arg.split_whitespace().collect();

        let output = Command::new(name)
            .args(&args)
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Some tools output version to stderr
        let version_output = if stdout.trim().is_empty() {
            stderr.to_string()
        } else {
            stdout.to_string()
        };

        // Extract first line and clean it up
        let first_line = version_output.lines().next()?;
        Some(first_line.trim().to_string())
    }

    /// Get custom PATH entries (non-standard paths)
    pub fn get_custom_paths(&self) -> Vec<PathBuf> {
        let standard_paths: HashSet<&str> = [
            "/usr/bin",
            "/usr/local/bin",
            "/bin",
            "/sbin",
            "/usr/sbin",
        ].into_iter().collect();

        std::env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .filter(|p| !p.is_empty() && !standard_paths.contains(p))
            .map(PathBuf::from)
            .collect()
    }

    /// Count tools by category
    pub fn count_by_category(tools: &[DetectedTool]) -> Vec<(ToolCategory, usize)> {
        use std::collections::HashMap;

        let mut counts: HashMap<ToolCategory, usize> = HashMap::new();
        for tool in tools {
            *counts.entry(tool.category).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_category_display() {
        assert_eq!(format!("{}", ToolCategory::VersionControl), "Version Control");
        assert_eq!(format!("{}", ToolCategory::ContainerRuntime), "Containers");
    }

    #[test]
    fn test_detected_tool_creation() {
        let tool = DetectedTool::new("git", PathBuf::from("/usr/bin/git"), ToolCategory::VersionControl)
            .with_version("git version 2.40.0");

        assert_eq!(tool.name, "git");
        assert_eq!(tool.category, ToolCategory::VersionControl);
        assert!(tool.version.is_some());
    }

    #[test]
    fn test_custom_paths() {
        let analyzer = ToolsAnalyzer::new();
        let custom = analyzer.get_custom_paths();

        // Custom paths should not include standard system paths
        for path in &custom {
            let path_str = path.to_string_lossy();
            assert!(!path_str.starts_with("/usr/bin"));
        }
    }

    #[test]
    fn test_count_by_category() {
        let tools = vec![
            DetectedTool::new("git", PathBuf::from("/usr/bin/git"), ToolCategory::VersionControl),
            DetectedTool::new("hg", PathBuf::from("/usr/bin/hg"), ToolCategory::VersionControl),
            DetectedTool::new("docker", PathBuf::from("/usr/bin/docker"), ToolCategory::ContainerRuntime),
        ];

        let counts = ToolsAnalyzer::count_by_category(&tools);

        assert!(!counts.is_empty());
        // VersionControl should have count of 2
        let vc_count = counts.iter().find(|(cat, _)| *cat == ToolCategory::VersionControl);
        assert_eq!(vc_count.map(|(_, c)| *c), Some(2));
    }
}
