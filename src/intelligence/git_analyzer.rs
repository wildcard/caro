//! Git repository state analyzer
//!
//! Extracts git context including branch, uncommitted changes, and remote status.

use super::ContextError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Git repository context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    /// Whether directory is a git repository
    pub is_repo: bool,
    /// Current branch name
    pub branch: Option<String>,
    /// Number of uncommitted changes
    pub uncommitted_changes: usize,
    /// Number of staged changes
    pub staged_changes: usize,
    /// Number of commits ahead of remote
    pub ahead: usize,
    /// Number of commits behind remote
    pub behind: usize,
    /// Last commit message (first line)
    pub last_commit: Option<String>,
    /// Whether there are untracked files
    pub has_untracked: bool,
}

impl GitContext {
    /// Create empty/non-repo context
    pub fn not_a_repo() -> Self {
        Self {
            is_repo: false,
            branch: None,
            uncommitted_changes: 0,
            staged_changes: 0,
            ahead: 0,
            behind: 0,
            last_commit: None,
            has_untracked: false,
        }
    }

    /// Convert to LLM-friendly context string
    pub fn to_llm_context(&self) -> String {
        if !self.is_repo {
            return "Git: Not a repository".to_string();
        }

        let mut lines = Vec::new();

        if let Some(branch) = &self.branch {
            lines.push(format!("Git Branch: {}", branch));
        }

        if self.staged_changes > 0 {
            lines.push(format!("Staged Changes: {}", self.staged_changes));
        }

        if self.uncommitted_changes > 0 {
            lines.push(format!("Uncommitted Changes: {}", self.uncommitted_changes));
        }

        if self.has_untracked {
            lines.push("Has Untracked Files: yes".to_string());
        }

        if self.ahead > 0 {
            lines.push(format!("Ahead of Remote: {} commits", self.ahead));
        }

        if self.behind > 0 {
            lines.push(format!("Behind Remote: {} commits", self.behind));
        }

        if let Some(commit) = &self.last_commit {
            lines.push(format!("Last Commit: {}", commit));
        }

        if lines.is_empty() {
            "Git: Clean working directory".to_string()
        } else {
            lines.join("\n")
        }
    }
}

/// Git repository analyzer
pub struct GitAnalyzer;

impl GitAnalyzer {
    /// Analyze git repository state at the given path
    pub async fn analyze(path: &Path) -> Result<GitContext, ContextError> {
        // Check if git is available
        if !Self::is_git_available().await {
            return Ok(GitContext::not_a_repo());
        }

        // Check if path is a git repository
        if !Self::is_git_repo(path).await {
            return Ok(GitContext::not_a_repo());
        }

        // Build git context
        let branch = Self::get_current_branch(path).await.ok();
        let (staged, uncommitted, has_untracked) = Self::get_change_counts(path).await;
        let (ahead, behind) = Self::get_remote_status(path).await;
        let last_commit = Self::get_last_commit(path).await.ok();

        Ok(GitContext {
            is_repo: true,
            branch,
            uncommitted_changes: uncommitted,
            staged_changes: staged,
            ahead,
            behind,
            last_commit,
            has_untracked,
        })
    }

    async fn is_git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .is_ok()
    }

    async fn is_git_repo(path: &Path) -> bool {
        Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    async fn get_current_branch(path: &Path) -> Result<String, ContextError> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .current_dir(path)
            .output()
            .await?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(branch)
        } else {
            Err(ContextError::GitError {
                message: "Failed to get current branch".to_string(),
            })
        }
    }

    async fn get_change_counts(path: &Path) -> (usize, usize, bool) {
        let output = match Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(path)
            .output()
            .await
        {
            Ok(output) => output,
            Err(_) => return (0, 0, false),
        };

        if !output.status.success() {
            return (0, 0, false);
        }

        let status_output = String::from_utf8_lossy(&output.stdout);
        let mut staged = 0;
        let mut uncommitted = 0;
        let mut has_untracked = false;

        for line in status_output.lines() {
            if line.len() < 2 {
                continue;
            }

            let index_status = &line[0..1];
            let worktree_status = &line[1..2];

            // Staged changes (index)
            if index_status != " " && index_status != "?" {
                staged += 1;
            }

            // Uncommitted changes (worktree)
            if worktree_status != " " && worktree_status != "?" {
                uncommitted += 1;
            }

            // Untracked files
            if index_status == "?" {
                has_untracked = true;
            }
        }

        (staged, uncommitted, has_untracked)
    }

    async fn get_remote_status(path: &Path) -> (usize, usize) {
        // Get remote tracking branch status
        let output = match Command::new("git")
            .arg("rev-list")
            .arg("--left-right")
            .arg("--count")
            .arg("HEAD...@{upstream}")
            .current_dir(path)
            .stderr(Stdio::null())
            .output()
            .await
        {
            Ok(output) => output,
            Err(_) => return (0, 0),
        };

        if !output.status.success() {
            return (0, 0);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.trim().split_whitespace().collect();

        if parts.len() >= 2 {
            let ahead = parts[0].parse().unwrap_or(0);
            let behind = parts[1].parse().unwrap_or(0);
            (ahead, behind)
        } else {
            (0, 0)
        }
    }

    async fn get_last_commit(path: &Path) -> Result<String, ContextError> {
        let output = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--pretty=%s")
            .current_dir(path)
            .output()
            .await?;

        if output.status.success() {
            let commit = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(commit)
        } else {
            Err(ContextError::GitError {
                message: "Failed to get last commit".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_git_context_not_a_repo() {
        let ctx = GitContext::not_a_repo();
        assert!(!ctx.is_repo);
        assert!(ctx.branch.is_none());
    }

    #[test]
    fn test_git_context_to_llm_string_not_repo() {
        let ctx = GitContext::not_a_repo();
        let llm_str = ctx.to_llm_context();
        assert_eq!(llm_str, "Git: Not a repository");
    }

    #[test]
    fn test_git_context_to_llm_string_clean() {
        let ctx = GitContext {
            is_repo: true,
            branch: Some("main".to_string()),
            uncommitted_changes: 0,
            staged_changes: 0,
            ahead: 0,
            behind: 0,
            last_commit: None,
            has_untracked: false,
        };
        let llm_str = ctx.to_llm_context();
        assert!(llm_str.contains("main"));
    }

    #[tokio::test]
    async fn test_analyze_current_repo() {
        // cmdai is a git repository
        let cwd = env::current_dir().unwrap();
        let result = GitAnalyzer::analyze(&cwd).await;
        assert!(result.is_ok());
        let ctx = result.unwrap();
        // Should detect as a git repo
        assert!(ctx.is_repo);
    }

    #[tokio::test]
    async fn test_is_git_available() {
        // git should be available in dev environment
        let available = GitAnalyzer::is_git_available().await;
        assert!(available);
    }

    #[tokio::test]
    async fn test_non_git_directory() {
        let tmp_dir = std::env::temp_dir();
        let result = GitAnalyzer::analyze(&tmp_dir).await;
        assert!(result.is_ok());
        // /tmp is typically not a git repo
        // (but might be in some environments, so we just check it doesn't error)
    }
}
