//! Environment and context analysis

use super::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// Insights about the current environment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentInsights {
    /// Detected project type in current directory
    pub project_type: Option<ProjectType>,

    /// Git repository state if in a git repo
    pub git_state: Option<GitState>,

    /// Notable environment variables (non-sensitive)
    pub notable_env_vars: Vec<String>,

    /// Custom PATH entries
    pub custom_paths: Vec<PathBuf>,

    /// Current working directory
    pub current_dir: Option<PathBuf>,
}

/// Type of project detected in current directory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Rust project (Cargo.toml)
    Rust,

    /// Node.js project (package.json)
    Node,

    /// Python project (pyproject.toml, setup.py, requirements.txt)
    Python,

    /// Go project (go.mod)
    Go,

    /// Just a git repository
    Git,

    /// Docker project (Dockerfile, docker-compose.yml)
    Docker,

    /// Unknown project type
    Unknown,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::Node => write!(f, "Node.js"),
            Self::Python => write!(f, "Python"),
            Self::Go => write!(f, "Go"),
            Self::Git => write!(f, "Git repository"),
            Self::Docker => write!(f, "Docker"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Git repository state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    /// Current branch name
    pub branch: String,

    /// Is working tree clean?
    pub is_clean: bool,

    /// Number of staged files
    pub staged_count: usize,

    /// Number of modified (unstaged) files
    pub modified_count: usize,

    /// Number of untracked files
    pub untracked_count: usize,

    /// Commits ahead of remote
    pub ahead: usize,

    /// Commits behind remote
    pub behind: usize,

    /// Is there an ongoing merge/rebase?
    pub has_conflicts: bool,
}

impl GitState {
    /// Get a summary of the git state for suggestions
    pub fn summary(&self) -> String {
        if self.has_conflicts {
            "merge/rebase in progress".to_string()
        } else if !self.is_clean {
            let mut parts = Vec::new();
            if self.staged_count > 0 {
                parts.push(format!("{} staged", self.staged_count));
            }
            if self.modified_count > 0 {
                parts.push(format!("{} modified", self.modified_count));
            }
            if self.untracked_count > 0 {
                parts.push(format!("{} untracked", self.untracked_count));
            }
            parts.join(", ")
        } else if self.ahead > 0 {
            format!("{} commits to push", self.ahead)
        } else {
            "clean".to_string()
        }
    }
}

/// Analyzes the current environment
pub struct EnvironmentAnalyzer;

impl Default for EnvironmentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentAnalyzer {
    /// Create a new environment analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze the current environment
    pub async fn analyze(&self) -> Result<EnvironmentInsights> {
        let current_dir = std::env::current_dir().ok();

        let project_type = if let Some(ref dir) = current_dir {
            self.detect_project_type(dir)
        } else {
            None
        };

        let git_state = if let Some(ref dir) = current_dir {
            self.detect_git_state(dir).await
        } else {
            None
        };

        let notable_env_vars = self.get_notable_env_vars();
        let custom_paths = self.get_custom_paths();

        Ok(EnvironmentInsights {
            project_type,
            git_state,
            notable_env_vars,
            custom_paths,
            current_dir,
        })
    }

    /// Detect project type from directory contents
    fn detect_project_type(&self, dir: &PathBuf) -> Option<ProjectType> {
        // Check for Rust
        if dir.join("Cargo.toml").exists() {
            return Some(ProjectType::Rust);
        }

        // Check for Node.js
        if dir.join("package.json").exists() {
            return Some(ProjectType::Node);
        }

        // Check for Python
        if dir.join("pyproject.toml").exists()
            || dir.join("setup.py").exists()
            || dir.join("requirements.txt").exists()
        {
            return Some(ProjectType::Python);
        }

        // Check for Go
        if dir.join("go.mod").exists() {
            return Some(ProjectType::Go);
        }

        // Check for Docker
        if dir.join("Dockerfile").exists() || dir.join("docker-compose.yml").exists() {
            return Some(ProjectType::Docker);
        }

        // Check for Git (fallback)
        if dir.join(".git").exists() {
            return Some(ProjectType::Git);
        }

        None
    }

    /// Detect git repository state
    async fn detect_git_state(&self, dir: &PathBuf) -> Option<GitState> {
        // Check if this is a git repo
        if !dir.join(".git").exists() {
            return None;
        }

        // Get current branch
        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

        // Get status
        let status_output = Command::new("git")
            .args(["status", "--porcelain=v1"])
            .current_dir(dir)
            .output()
            .ok()
            .filter(|o| o.status.success())?;

        let status = String::from_utf8_lossy(&status_output.stdout);
        let lines: Vec<&str> = status.lines().collect();

        let mut staged_count = 0;
        let mut modified_count = 0;
        let mut untracked_count = 0;
        let mut has_conflicts = false;

        for line in &lines {
            if line.len() < 2 {
                continue;
            }
            let chars: Vec<char> = line.chars().collect();
            let index_status = chars[0];
            let worktree_status = chars[1];

            // Check for conflicts
            if index_status == 'U' || worktree_status == 'U' {
                has_conflicts = true;
            }

            // Count staged (index has changes)
            if index_status != ' ' && index_status != '?' && index_status != 'U' {
                staged_count += 1;
            }

            // Count modified (worktree has changes)
            if worktree_status == 'M' || worktree_status == 'D' {
                modified_count += 1;
            }

            // Count untracked
            if index_status == '?' {
                untracked_count += 1;
            }
        }

        // Get ahead/behind counts
        let (ahead, behind) = self.get_ahead_behind(dir);

        Some(GitState {
            branch,
            is_clean: lines.is_empty(),
            staged_count,
            modified_count,
            untracked_count,
            ahead,
            behind,
            has_conflicts,
        })
    }

    /// Get commits ahead/behind remote
    fn get_ahead_behind(&self, dir: &PathBuf) -> (usize, usize) {
        let output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "@{upstream}...HEAD"])
            .current_dir(dir)
            .output()
            .ok();

        if let Some(output) = output {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = text.trim().split('\t').collect();
                if parts.len() == 2 {
                    let behind = parts[0].parse().unwrap_or(0);
                    let ahead = parts[1].parse().unwrap_or(0);
                    return (ahead, behind);
                }
            }
        }

        (0, 0)
    }

    /// Get notable (non-sensitive) environment variables
    fn get_notable_env_vars(&self) -> Vec<String> {
        let notable_vars = [
            "EDITOR",
            "VISUAL",
            "TERM",
            "SHELL",
            "LANG",
            "LC_ALL",
            "VIRTUAL_ENV",
            "CONDA_DEFAULT_ENV",
            "NVM_DIR",
            "GOPATH",
            "CARGO_HOME",
            "RUSTUP_HOME",
        ];

        notable_vars
            .iter()
            .filter_map(|&var| {
                std::env::var(var).ok().map(|val| format!("{}={}", var, val))
            })
            .collect()
    }

    /// Get custom PATH entries
    fn get_custom_paths(&self) -> Vec<PathBuf> {
        let standard_paths = [
            "/usr/bin",
            "/usr/local/bin",
            "/bin",
            "/sbin",
            "/usr/sbin",
        ];

        std::env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .filter(|p| !p.is_empty() && !standard_paths.contains(p))
            .map(PathBuf::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_display() {
        assert_eq!(format!("{}", ProjectType::Rust), "Rust");
        assert_eq!(format!("{}", ProjectType::Node), "Node.js");
    }

    #[test]
    fn test_git_state_summary_clean() {
        let state = GitState {
            branch: "main".to_string(),
            is_clean: true,
            staged_count: 0,
            modified_count: 0,
            untracked_count: 0,
            ahead: 0,
            behind: 0,
            has_conflicts: false,
        };

        assert_eq!(state.summary(), "clean");
    }

    #[test]
    fn test_git_state_summary_changes() {
        let state = GitState {
            branch: "feature".to_string(),
            is_clean: false,
            staged_count: 2,
            modified_count: 3,
            untracked_count: 1,
            ahead: 0,
            behind: 0,
            has_conflicts: false,
        };

        let summary = state.summary();
        assert!(summary.contains("2 staged"));
        assert!(summary.contains("3 modified"));
        assert!(summary.contains("1 untracked"));
    }

    #[test]
    fn test_git_state_summary_ahead() {
        let state = GitState {
            branch: "main".to_string(),
            is_clean: true,
            staged_count: 0,
            modified_count: 0,
            untracked_count: 0,
            ahead: 3,
            behind: 0,
            has_conflicts: false,
        };

        assert_eq!(state.summary(), "3 commits to push");
    }

    #[test]
    fn test_environment_analyzer_creation() {
        let analyzer = EnvironmentAnalyzer::new();
        // Just verify it can be created
        let _ = analyzer;
    }
}
