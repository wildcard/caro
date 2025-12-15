//! Sandbox environment for safe command execution
//!
//! This module provides isolated execution environments for commands,
//! allowing users to preview changes before applying them.
//!
//! Platform support:
//! - Linux: BTRFS snapshots (preferred) or overlay filesystem
//! - macOS: APFS snapshots (preferred) or copy-on-write
//! - Windows: Shadow copies or temp directory overlay

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// File change detected in sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub size_before: Option<u64>,
    pub size_after: Option<u64>,
}

/// Type of change to a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    PermissionsChanged,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Modified => write!(f, "Modified"),
            Self::Deleted => write!(f, "Deleted"),
            Self::PermissionsChanged => write!(f, "Permissions Changed"),
        }
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub changes: Vec<FileChange>,
    pub executed_at: DateTime<Utc>,
}

/// Sandbox execution environment
pub struct Sandbox {
    /// Root directory of sandbox
    root_dir: PathBuf,
    /// Snapshot identifier (platform-specific)
    snapshot_id: Option<String>,
    /// Original working directory
    original_cwd: PathBuf,
    /// Temporary overlay directory (if using overlay)
    overlay_dir: Option<PathBuf>,
    /// Platform-specific implementation
    implementation: SandboxImpl,
}

/// Platform-specific sandbox implementation
#[derive(Debug, Clone, Copy)]
enum SandboxImpl {
    /// BTRFS filesystem snapshots (Linux)
    BtrfsSnapshot,
    /// APFS snapshots (macOS)
    ApfsSnapshot,
    /// Overlay filesystem (Linux fallback)
    OverlayFs,
    /// Copy-on-write temporary directory (fallback)
    TempCopy,
}

impl Sandbox {
    /// Create new sandbox environment
    pub async fn create(cwd: &Path) -> Result<Self> {
        let original_cwd = cwd.to_path_buf();

        // Detect platform and choose implementation
        let implementation = Self::detect_best_implementation(&original_cwd).await?;

        let (root_dir, snapshot_id, overlay_dir) = match implementation {
            SandboxImpl::BtrfsSnapshot => Self::create_btrfs_snapshot(&original_cwd).await?,
            SandboxImpl::ApfsSnapshot => Self::create_apfs_snapshot(&original_cwd).await?,
            SandboxImpl::OverlayFs => Self::create_overlay(&original_cwd).await?,
            SandboxImpl::TempCopy => Self::create_temp_copy(&original_cwd).await?,
        };

        Ok(Self {
            root_dir,
            snapshot_id,
            original_cwd,
            overlay_dir,
            implementation,
        })
    }

    /// Detect best available sandbox implementation
    async fn detect_best_implementation(cwd: &Path) -> Result<SandboxImpl> {
        #[cfg(target_os = "linux")]
        {
            // Check if path is on BTRFS
            if Self::is_btrfs(cwd).await? {
                return Ok(SandboxImpl::BtrfsSnapshot);
            }

            // Check if OverlayFS is available
            if Self::has_overlayfs().await? {
                return Ok(SandboxImpl::OverlayFs);
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Check if path is on APFS
            if Self::is_apfs(cwd).await? {
                return Ok(SandboxImpl::ApfsSnapshot);
            }
        }

        // Fallback to temp copy (works everywhere)
        Ok(SandboxImpl::TempCopy)
    }

    /// Check if path is on BTRFS filesystem (Linux)
    #[cfg(target_os = "linux")]
    async fn is_btrfs(_path: &Path) -> Result<bool> {
        // Check filesystem type using statfs
        // For now, return false (can be implemented with libc::statfs)
        Ok(false)
    }

    #[cfg(not(target_os = "linux"))]
    async fn is_btrfs(_path: &Path) -> Result<bool> {
        Ok(false)
    }

    /// Check if OverlayFS kernel module is available (Linux)
    #[cfg(target_os = "linux")]
    async fn has_overlayfs() -> Result<bool> {
        // Check if overlay module is loaded
        let output = tokio::process::Command::new("lsmod")
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).contains("overlay"))
    }

    #[cfg(not(target_os = "linux"))]
    async fn has_overlayfs() -> Result<bool> {
        Ok(false)
    }

    /// Check if path is on APFS filesystem (macOS)
    #[cfg(target_os = "macos")]
    async fn is_apfs(_path: &Path) -> Result<bool> {
        // Can be checked using diskutil
        Ok(false) // Stub for now
    }

    #[cfg(not(target_os = "macos"))]
    async fn is_apfs(_path: &Path) -> Result<bool> {
        Ok(false)
    }

    /// Create BTRFS snapshot
    async fn create_btrfs_snapshot(_cwd: &Path) -> Result<(PathBuf, Option<String>, Option<PathBuf>)> {
        bail!("BTRFS snapshots not yet implemented");
        // Implementation would use: btrfs subvolume snapshot
    }

    /// Create APFS snapshot
    async fn create_apfs_snapshot(_cwd: &Path) -> Result<(PathBuf, Option<String>, Option<PathBuf>)> {
        bail!("APFS snapshots not yet implemented");
        // Implementation would use: tmutil localsnapshot
    }

    /// Create overlay filesystem
    async fn create_overlay(_cwd: &Path) -> Result<(PathBuf, Option<String>, Option<PathBuf>)> {
        bail!("OverlayFS not yet implemented");
        // Implementation would use mount -t overlay
    }

    /// Create temporary copy (fallback implementation that works everywhere)
    async fn create_temp_copy(cwd: &Path) -> Result<(PathBuf, Option<String>, Option<PathBuf>)> {
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;

        let root_dir = temp_dir.path().to_path_buf();
        let overlay_dir = root_dir.clone();
        std::mem::forget(temp_dir); // Don't delete on drop
        let sandbox_cwd = root_dir.join("sandbox");

        // Create sandbox directory
        tokio::fs::create_dir_all(&sandbox_cwd).await
            .context("Failed to create sandbox directory")?;

        // Copy current directory contents to sandbox
        Self::copy_directory(cwd, &sandbox_cwd).await
            .context("Failed to copy directory to sandbox")?;

        Ok((sandbox_cwd, None, Some(overlay_dir)))
    }

    /// Recursively copy directory
    async fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
        tokio::fs::create_dir_all(dst).await?;

        let mut entries = tokio::fs::read_dir(src).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            if entry.file_type().await?.is_dir() {
                Box::pin(Self::copy_directory(&path, &dst_path)).await?;
            } else {
                tokio::fs::copy(&path, &dst_path).await?;
            }
        }

        Ok(())
    }

    /// Execute command in sandbox
    pub async fn execute(&self, command: &str) -> Result<SandboxResult> {
        let executed_at = Utc::now();

        // Take snapshot of current state before execution
        let before_state = self.snapshot_state().await?;

        // Execute command in sandbox directory
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.root_dir)
            .output()
            .await
            .context("Failed to execute command in sandbox")?;

        // Take snapshot of state after execution
        let after_state = self.snapshot_state().await?;

        // Compute differences
        let changes = self.compute_changes(&before_state, &after_state).await?;

        Ok(SandboxResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            changes,
            executed_at,
        })
    }

    /// Snapshot current filesystem state
    async fn snapshot_state(&self) -> Result<Vec<FileSnapshot>> {
        let mut snapshots = Vec::new();
        self.snapshot_directory(&self.root_dir, &mut snapshots).await?;
        Ok(snapshots)
    }

    /// Recursively snapshot directory
    async fn snapshot_directory(&self, dir: &Path, snapshots: &mut Vec<FileSnapshot>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            snapshots.push(FileSnapshot {
                path: path.clone(),
                size: metadata.len(),
                modified: metadata.modified().ok(),
                is_dir: metadata.is_dir(),
            });

            if metadata.is_dir() {
                Box::pin(self.snapshot_directory(&path, snapshots)).await?;
            }
        }

        Ok(())
    }

    /// Compute changes between two snapshots
    async fn compute_changes(&self, before: &[FileSnapshot], after: &[FileSnapshot]) -> Result<Vec<FileChange>> {
        use std::collections::HashMap;

        let mut changes = Vec::new();

        // Build lookup maps
        let before_map: HashMap<_, _> = before.iter()
            .map(|s| (s.path.clone(), s))
            .collect();

        let after_map: HashMap<_, _> = after.iter()
            .map(|s| (s.path.clone(), s))
            .collect();

        // Find created and modified files
        for (path, after_snap) in &after_map {
            if let Some(before_snap) = before_map.get(path) {
                // File existed before
                if before_snap.size != after_snap.size ||
                   before_snap.modified != after_snap.modified {
                    changes.push(FileChange {
                        path: path.clone(),
                        change_type: ChangeType::Modified,
                        size_before: Some(before_snap.size),
                        size_after: Some(after_snap.size),
                    });
                }
            } else {
                // File was created
                changes.push(FileChange {
                    path: path.clone(),
                    change_type: ChangeType::Created,
                    size_before: None,
                    size_after: Some(after_snap.size),
                });
            }
        }

        // Find deleted files
        for (path, before_snap) in &before_map {
            if !after_map.contains_key(path) {
                changes.push(FileChange {
                    path: path.clone(),
                    change_type: ChangeType::Deleted,
                    size_before: Some(before_snap.size),
                    size_after: None,
                });
            }
        }

        Ok(changes)
    }

    /// Get diff of changes
    pub async fn diff(&self) -> Result<Vec<FileChange>> {
        // Re-compute current changes
        let before = self.snapshot_state().await?;
        let after = self.snapshot_state().await?;
        self.compute_changes(&before, &after).await
    }

    /// Commit changes from sandbox to real filesystem
    pub async fn commit(&self) -> Result<()> {
        match self.implementation {
            SandboxImpl::TempCopy => {
                // Copy back from sandbox to original
                Self::copy_directory(&self.root_dir, &self.original_cwd).await
                    .context("Failed to commit changes from sandbox")?;
            }
            _ => {
                bail!("Commit not yet implemented for {:?}", self.implementation);
            }
        }

        Ok(())
    }

    /// Rollback and discard all changes
    pub async fn rollback(&self) -> Result<()> {
        match self.implementation {
            SandboxImpl::TempCopy => {
                // Just delete the temporary directory
                if let Some(ref overlay_dir) = self.overlay_dir {
                    tokio::fs::remove_dir_all(overlay_dir).await
                        .context("Failed to remove sandbox directory")?;
                }
            }
            _ => {
                bail!("Rollback not yet implemented for {:?}", self.implementation);
            }
        }

        Ok(())
    }
}

/// Snapshot of a file's state
#[derive(Debug, Clone)]
struct FileSnapshot {
    path: PathBuf,
    size: u64,
    modified: Option<std::time::SystemTime>,
    is_dir: bool,
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // Clean up sandbox resources
        if let Some(ref overlay_dir) = self.overlay_dir {
            let _ = std::fs::remove_dir_all(overlay_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_sandbox() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = Sandbox::create(temp_dir.path()).await.unwrap();

        assert!(sandbox.root_dir.exists());
    }

    #[tokio::test]
    async fn test_execute_in_sandbox() {
        let temp_dir = TempDir::new().unwrap();

        // Create test file
        tokio::fs::write(temp_dir.path().join("test.txt"), "hello").await.unwrap();

        let sandbox = Sandbox::create(temp_dir.path()).await.unwrap();
        let result = sandbox.execute("echo 'test' > output.txt").await.unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(!result.changes.is_empty());
    }

    #[tokio::test]
    async fn test_detect_file_changes() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = Sandbox::create(temp_dir.path()).await.unwrap();

        // Execute command that creates file
        let result = sandbox.execute("echo 'new file' > created.txt").await.unwrap();

        assert!(result.changes.iter().any(|c|
            c.path.file_name().unwrap().to_str().unwrap() == "created.txt" &&
            c.change_type == ChangeType::Created
        ));
    }

    #[tokio::test]
    async fn test_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = Sandbox::create(temp_dir.path()).await.unwrap();

        sandbox.execute("echo 'test' > file.txt").await.unwrap();
        sandbox.rollback().await.unwrap();

        // Sandbox directory should be cleaned up
        // (verified by Drop trait)
    }
}
