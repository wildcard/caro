//! Config file editor with backup support
//!
//! Safely modifies shell configuration files with timestamped backups
//! and atomic writes.

use super::types::InstallationError;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Configuration file editor with backup support
pub struct ConfigEditor {
    /// Directory for storing backups
    backup_dir: PathBuf,

    /// Map of backup labels to their paths
    backups: HashMap<String, PathBuf>,

    /// Whether to use atomic writes
    atomic_writes: bool,
}

impl ConfigEditor {
    /// Create a new config editor
    pub fn new() -> Result<Self, InstallationError> {
        let backup_dir = dirs::cache_dir()
            .ok_or_else(|| InstallationError::BackupFailed("No cache directory".into()))?
            .join("caro")
            .join("backups");

        Ok(Self {
            backup_dir,
            backups: HashMap::new(),
            atomic_writes: true,
        })
    }

    /// Create with a custom backup directory
    pub fn with_backup_dir(backup_dir: PathBuf) -> Self {
        Self {
            backup_dir,
            backups: HashMap::new(),
            atomic_writes: true,
        }
    }

    /// Disable atomic writes (for testing)
    pub fn without_atomic_writes(mut self) -> Self {
        self.atomic_writes = false;
        self
    }

    /// Ensure the backup directory exists
    fn ensure_backup_dir(&self) -> Result<(), InstallationError> {
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir)?;
        }
        Ok(())
    }

    /// Create a timestamped backup of a file
    pub fn backup(&mut self, path: &Path, label: &str) -> Result<PathBuf, InstallationError> {
        self.ensure_backup_dir()?;

        // Expand tilde if present
        let expanded_path = expand_tilde(path);

        if !expanded_path.exists() {
            // No file to backup, store empty indicator
            self.backups.insert(label.to_string(), PathBuf::new());
            return Ok(PathBuf::new());
        }

        // Create timestamped backup filename
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let original_name = expanded_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let backup_name = format!("{}.{}.{}", label, original_name, timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        // Copy the file
        fs::copy(&expanded_path, &backup_path)?;

        // Store the backup reference
        self.backups.insert(label.to_string(), backup_path.clone());

        Ok(backup_path)
    }

    /// Restore a backup
    pub fn restore_backup(&self, label: &str) -> Result<(), InstallationError> {
        let backup_path = self.backups.get(label)
            .ok_or_else(|| InstallationError::BackupFailed(format!("No backup with label '{}'", label)))?;

        // If backup path is empty, the original file didn't exist
        if backup_path.as_os_str().is_empty() {
            return Ok(());
        }

        // Find the original path from the backup name
        // This is a simplification - in production we'd store this mapping
        Err(InstallationError::BackupFailed(
            "Restore requires original path mapping".into()
        ))
    }

    /// Get the path to a backup by label
    pub fn get_backup_path(&self, label: &str) -> Option<&PathBuf> {
        self.backups.get(label)
    }

    /// Read a config file
    pub fn read(&self, path: &Path) -> Result<String, InstallationError> {
        let expanded_path = expand_tilde(path);
        let mut file = fs::File::open(&expanded_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Write a config file atomically
    pub fn write(&self, path: &Path, content: &str) -> Result<(), InstallationError> {
        let expanded_path = expand_tilde(path);

        if self.atomic_writes {
            self.write_atomic(&expanded_path, content)
        } else {
            self.write_direct(&expanded_path, content)
        }
    }

    /// Write atomically using temp file + rename
    fn write_atomic(&self, path: &Path, content: &str) -> Result<(), InstallationError> {
        let temp_path = path.with_extension("tmp.caro");

        // Write to temp file
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        // Preserve original file permissions if the file exists
        #[cfg(unix)]
        if path.exists() {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(path)?;
            let permissions = metadata.permissions();
            fs::set_permissions(&temp_path, permissions)?;
        }

        // Atomic rename
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Write directly (non-atomic, for testing)
    fn write_direct(&self, path: &Path, content: &str) -> Result<(), InstallationError> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Add a line to a config file if not already present
    pub fn add_line_if_missing(
        &self,
        path: &Path,
        line: &str,
        skip_if_contains: Option<&str>,
    ) -> Result<bool, InstallationError> {
        let expanded_path = expand_tilde(path);

        // Read existing content (or empty if file doesn't exist)
        let content = if expanded_path.exists() {
            self.read(&expanded_path)?
        } else {
            String::new()
        };

        // Check if we should skip
        if let Some(pattern) = skip_if_contains {
            if content.contains(pattern) {
                return Ok(false); // Already present
            }
        } else if content.contains(line) {
            return Ok(false); // Already present
        }

        // Add the line
        let new_content = if content.is_empty() || content.ends_with('\n') {
            format!("{}{}\n", content, line)
        } else {
            format!("{}\n{}\n", content, line)
        };

        self.write(&expanded_path, &new_content)?;
        Ok(true)
    }

    /// Replace a pattern in a config file
    pub fn replace_pattern(
        &self,
        path: &Path,
        pattern: &str,
        replacement: &str,
    ) -> Result<bool, InstallationError> {
        let expanded_path = expand_tilde(path);

        if !expanded_path.exists() {
            return Ok(false);
        }

        let content = self.read(&expanded_path)?;

        if !content.contains(pattern) {
            return Ok(false);
        }

        let new_content = content.replace(pattern, replacement);
        self.write(&expanded_path, &new_content)?;

        Ok(true)
    }

    /// Check if a file contains a pattern
    pub fn contains(&self, path: &Path, pattern: &str) -> Result<bool, InstallationError> {
        let expanded_path = expand_tilde(path);

        if !expanded_path.exists() {
            return Ok(false);
        }

        let content = self.read(&expanded_path)?;
        Ok(content.contains(pattern))
    }

    /// List all backups
    pub fn list_backups(&self) -> Result<Vec<(String, PathBuf)>, InstallationError> {
        let mut backups = Vec::new();

        if !self.backup_dir.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Extract label from filename (format: label.filename.timestamp)
                    let parts: Vec<&str> = name.splitn(2, '.').collect();
                    if !parts.is_empty() {
                        backups.push((parts[0].to_string(), path));
                    }
                }
            }
        }

        Ok(backups)
    }

    /// Clean up old backups (keep only recent ones)
    pub fn cleanup_old_backups(&self, keep_count: usize) -> Result<usize, InstallationError> {
        if !self.backup_dir.exists() {
            return Ok(0);
        }

        let mut entries: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        // Sort by modification time (oldest first)
        entries.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            a_time.cmp(&b_time)
        });

        // Remove oldest entries if we have more than keep_count
        let mut removed = 0;
        if entries.len() > keep_count {
            for entry in entries.iter().take(entries.len() - keep_count) {
                if fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }
}

impl Default for ConfigEditor {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigEditor")
    }
}

/// Expand ~ to home directory
fn expand_tilde(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path_str[2..]);
        }
    } else if path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_editor() -> (ConfigEditor, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let editor = ConfigEditor::with_backup_dir(temp_dir.path().join("backups"));
        (editor, temp_dir)
    }

    #[test]
    fn test_backup_and_list() {
        let (mut editor, temp_dir) = create_test_editor();

        // Create a test file
        let test_file = temp_dir.path().join("test.conf");
        fs::write(&test_file, "original content").unwrap();

        // Create backup
        let backup_path = editor.backup(&test_file, "test-label").unwrap();
        assert!(!backup_path.as_os_str().is_empty());
        assert!(backup_path.exists());

        // Verify backup content
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "original content");
    }

    #[test]
    fn test_add_line_if_missing() {
        let (editor, temp_dir) = create_test_editor();

        let test_file = temp_dir.path().join("config");
        fs::write(&test_file, "line1\n").unwrap();

        // Add new line
        let added = editor.add_line_if_missing(&test_file, "line2", None).unwrap();
        assert!(added);

        // Try to add same line again
        let added_again = editor.add_line_if_missing(&test_file, "line2", None).unwrap();
        assert!(!added_again);

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("line1"));
        assert!(content.contains("line2"));
    }

    #[test]
    fn test_add_line_with_skip_pattern() {
        let (editor, temp_dir) = create_test_editor();

        let test_file = temp_dir.path().join("config");
        fs::write(&test_file, "export FOO=bar\n").unwrap();

        // Skip if pattern exists
        let added = editor.add_line_if_missing(
            &test_file,
            "export FOO=baz",
            Some("FOO="),
        ).unwrap();
        assert!(!added);
    }

    #[test]
    fn test_replace_pattern() {
        let (editor, temp_dir) = create_test_editor();

        let test_file = temp_dir.path().join("config");
        fs::write(&test_file, "value=old\n").unwrap();

        let replaced = editor.replace_pattern(&test_file, "old", "new").unwrap();
        assert!(replaced);

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("value=new"));
        assert!(!content.contains("old"));
    }

    #[test]
    fn test_contains() {
        let (editor, temp_dir) = create_test_editor();

        let test_file = temp_dir.path().join("config");
        fs::write(&test_file, "some content here").unwrap();

        assert!(editor.contains(&test_file, "content").unwrap());
        assert!(!editor.contains(&test_file, "missing").unwrap());
    }

    #[test]
    fn test_expand_tilde() {
        let home = dirs::home_dir().unwrap();

        let expanded = expand_tilde(Path::new("~/.zshrc"));
        assert_eq!(expanded, home.join(".zshrc"));

        let no_tilde = expand_tilde(Path::new("/etc/config"));
        assert_eq!(no_tilde, PathBuf::from("/etc/config"));
    }

    #[test]
    fn test_cleanup_old_backups() {
        let (editor, _temp_dir) = create_test_editor();

        // Create the backup directory
        fs::create_dir_all(&editor.backup_dir).unwrap();

        // Create some test backup files
        for i in 0..5 {
            let path = editor.backup_dir.join(format!("backup{}.conf.20240101", i));
            fs::write(&path, "content").unwrap();
        }

        // Cleanup, keeping only 2
        let removed = editor.cleanup_old_backups(2).unwrap();
        assert_eq!(removed, 3);

        // Count remaining files
        let remaining: Vec<_> = fs::read_dir(&editor.backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(remaining.len(), 2);
    }
}
