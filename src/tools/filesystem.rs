//! FileSystem Tool - Path validation, file inspection, and directory operations
//!
//! Provides context-gathering capabilities for filesystem operations during
//! command generation. These tools help validate assumptions about paths,
//! permissions, and file existence before generating commands.

use super::{
    ParameterType, StructuredData, Tool, ToolCallParams, ToolCategory, ToolData, ToolParameters,
    ToolResult,
};
use async_trait::async_trait;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Instant;

/// FileSystem tool for path and file operations
pub struct FileSystemTool {
    /// Maximum depth for directory listing
    max_depth: usize,
    /// Maximum entries to return in listings
    max_entries: usize,
}

impl Default for FileSystemTool {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_entries: 100,
        }
    }
}

impl FileSystemTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_max_entries(mut self, entries: usize) -> Self {
        self.max_entries = entries;
        self
    }

    /// Check if a path exists
    fn check_exists(&self, path: &str, follow_symlinks: bool) -> ToolResult {
        let start = Instant::now();
        let p = Path::new(path);

        let exists = if follow_symlinks {
            p.exists()
        } else {
            p.symlink_metadata().is_ok()
        };

        ToolResult::success(
            ToolData::Boolean(exists),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get file/directory metadata
    fn get_metadata(&self, path: &str) -> ToolResult {
        let start = Instant::now();
        let p = Path::new(path);

        match fs::metadata(p) {
            Ok(meta) => {
                let mut data = StructuredData::new("file_metadata");

                data = data
                    .with_field("path", path)
                    .with_field("exists", true)
                    .with_field("is_file", meta.is_file())
                    .with_field("is_dir", meta.is_dir())
                    .with_field("is_symlink", p.is_symlink())
                    .with_field("size_bytes", meta.len())
                    .with_field("readonly", meta.permissions().readonly());

                #[cfg(unix)]
                {
                    let mode = meta.permissions().mode();
                    data = data
                        .with_field("mode_octal", format!("{:o}", mode & 0o7777))
                        .with_field("is_executable", mode & 0o111 != 0);
                }

                ToolResult::success(
                    ToolData::Structured(data),
                    start.elapsed().as_millis() as u64,
                )
            }
            Err(e) => ToolResult::error(
                format!("Failed to get metadata: {}", e),
                start.elapsed().as_millis() as u64,
            ),
        }
    }

    /// Check read/write/execute permissions
    fn check_permissions(&self, path: &str, check_type: &str) -> ToolResult {
        let start = Instant::now();
        let p = Path::new(path);

        if !p.exists() {
            return ToolResult::error("Path does not exist", start.elapsed().as_millis() as u64);
        }

        let result = match check_type {
            "read" => self.can_read(p),
            "write" => self.can_write(p),
            "execute" => self.can_execute(p),
            "all" => {
                let mut data = StructuredData::new("permissions");
                data = data
                    .with_field("can_read", self.can_read(p))
                    .with_field("can_write", self.can_write(p))
                    .with_field("can_execute", self.can_execute(p));

                return ToolResult::success(
                    ToolData::Structured(data),
                    start.elapsed().as_millis() as u64,
                );
            }
            _ => {
                return ToolResult::error(
                    format!("Unknown permission type: {}", check_type),
                    start.elapsed().as_millis() as u64,
                )
            }
        };

        ToolResult::success(
            ToolData::Boolean(result),
            start.elapsed().as_millis() as u64,
        )
    }

    fn can_read(&self, path: &Path) -> bool {
        if path.is_dir() {
            fs::read_dir(path).is_ok()
        } else {
            fs::File::open(path).is_ok()
        }
    }

    fn can_write(&self, path: &Path) -> bool {
        if path.is_dir() {
            // Try to check if we can create a file in the directory
            let test_path = path.join(".caro_write_test");
            let result = fs::File::create(&test_path).is_ok();
            let _ = fs::remove_file(&test_path);
            result
        } else if path.exists() {
            // For existing files, check if we can open for writing
            fs::OpenOptions::new().write(true).open(path).is_ok()
        } else {
            // For non-existent files, check if parent is writable
            path.parent().map(|p| self.can_write(p)).unwrap_or(false)
        }
    }

    fn can_execute(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            path.metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            path.extension()
                .map(|ext| {
                    matches!(
                        ext.to_str().unwrap_or("").to_lowercase().as_str(),
                        "exe" | "bat" | "cmd" | "com"
                    )
                })
                .unwrap_or(false)
        }
    }

    /// List directory contents
    fn list_directory(&self, path: &str, depth: usize, pattern: Option<&str>) -> ToolResult {
        let start = Instant::now();
        let p = Path::new(path);

        if !p.is_dir() {
            return ToolResult::error(
                "Path is not a directory",
                start.elapsed().as_millis() as u64,
            );
        }

        let mut entries = Vec::new();
        let effective_depth = depth.min(self.max_depth);

        if let Err(e) = self.collect_entries(p, &mut entries, 0, effective_depth, pattern) {
            return ToolResult::error(
                format!("Failed to list directory: {}", e),
                start.elapsed().as_millis() as u64,
            );
        }

        // Truncate to max entries
        entries.truncate(self.max_entries);

        ToolResult::success(ToolData::List(entries), start.elapsed().as_millis() as u64)
    }

    fn collect_entries(
        &self,
        dir: &Path,
        entries: &mut Vec<String>,
        current_depth: usize,
        max_depth: usize,
        pattern: Option<&str>,
    ) -> std::io::Result<()> {
        if current_depth > max_depth || entries.len() >= self.max_entries {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            if entries.len() >= self.max_entries {
                break;
            }

            let entry = entry?;
            let path = entry.path();
            let name = path.to_string_lossy().to_string();

            // Apply pattern filter if specified
            if let Some(pat) = pattern {
                if !name.contains(pat) {
                    continue;
                }
            }

            entries.push(name);

            // Recurse into directories
            if path.is_dir() && current_depth < max_depth {
                self.collect_entries(&path, entries, current_depth + 1, max_depth, pattern)?;
            }
        }

        Ok(())
    }

    /// Resolve a path to its canonical form
    fn resolve_path(&self, path: &str, expand_home: bool) -> ToolResult {
        let start = Instant::now();

        let expanded = if expand_home && path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                path.replacen('~', home.to_string_lossy().as_ref(), 1)
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        };

        let p = Path::new(&expanded);

        match p.canonicalize() {
            Ok(canonical) => ToolResult::success(
                ToolData::String(canonical.to_string_lossy().to_string()),
                start.elapsed().as_millis() as u64,
            ),
            Err(_) => {
                // Path doesn't exist, return the expanded path
                ToolResult::success(
                    ToolData::String(expanded),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
    }

    /// Check if path is under a safe directory (not system-critical)
    fn is_safe_path(&self, path: &str) -> ToolResult {
        let start = Instant::now();

        let dangerous_prefixes = [
            "/bin",
            "/sbin",
            "/usr/bin",
            "/usr/sbin",
            "/etc",
            "/boot",
            "/lib",
            "/lib64",
            "/usr/lib",
            "/var/lib",
            "/root",
            "/sys",
            "/proc",
            "/dev",
        ];

        let p = Path::new(path);
        let canonical = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        let path_str = canonical.to_string_lossy();

        let is_dangerous = dangerous_prefixes
            .iter()
            .any(|prefix| path_str.starts_with(prefix));

        let mut data = StructuredData::new("path_safety");
        data = data
            .with_field("path", path)
            .with_field("canonical", path_str.to_string())
            .with_field("is_safe", !is_dangerous)
            .with_field("is_system_path", is_dangerous);

        if is_dangerous {
            let matched = dangerous_prefixes
                .iter()
                .find(|prefix| path_str.starts_with(*prefix))
                .unwrap_or(&"unknown");
            data = data.with_field("matched_prefix", *matched);
        }

        ToolResult::success(
            ToolData::Structured(data),
            start.elapsed().as_millis() as u64,
        )
    }
}

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "File system operations: check existence, permissions, list directories, resolve paths"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .with_required(
                "operation",
                ParameterType::String,
                "Operation to perform: exists, metadata, permissions, list, resolve, is_safe",
            )
            .with_required("path", ParameterType::Path, "The file or directory path")
            .with_optional(
                "follow_symlinks",
                ParameterType::Boolean,
                "Follow symbolic links (default: true)",
            )
            .with_optional(
                "permission_type",
                ParameterType::String,
                "Permission to check: read, write, execute, all",
            )
            .with_optional(
                "depth",
                ParameterType::Integer,
                "Directory listing depth (default: 1)",
            )
            .with_optional(
                "pattern",
                ParameterType::String,
                "Filter pattern for directory listing",
            )
            .with_optional(
                "expand_home",
                ParameterType::Boolean,
                "Expand ~ to home directory (default: true)",
            )
    }

    async fn execute(&self, params: &ToolCallParams) -> ToolResult {
        let start = Instant::now();

        let operation = match params.get_string("operation") {
            Some(op) => op,
            None => return ToolResult::error("Missing required parameter: operation", 0),
        };

        let path = match params.get_string("path") {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter: path", 0),
        };

        match operation {
            "exists" => {
                let follow = params.get_bool("follow_symlinks").unwrap_or(true);
                self.check_exists(path, follow)
            }
            "metadata" => self.get_metadata(path),
            "permissions" => {
                let perm_type = params.get_string("permission_type").unwrap_or("all");
                self.check_permissions(path, perm_type)
            }
            "list" => {
                let depth = params.get_int("depth").unwrap_or(1) as usize;
                let pattern = params.get_string("pattern");
                self.list_directory(path, depth, pattern)
            }
            "resolve" => {
                let expand = params.get_bool("expand_home").unwrap_or(true);
                self.resolve_path(path, expand)
            }
            "is_safe" => self.is_safe_path(path),
            _ => ToolResult::error(
                format!("Unknown operation: {}", operation),
                start.elapsed().as_millis() as u64,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filesystem_exists() {
        let tool = FileSystemTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "exists")
            .with_path("path", "/tmp");

        let result = tool.execute(&params).await;
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[tokio::test]
    async fn test_filesystem_not_exists() {
        let tool = FileSystemTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "exists")
            .with_path("path", "/nonexistent/path/that/does/not/exist");

        let result = tool.execute(&params).await;
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[tokio::test]
    async fn test_filesystem_metadata() {
        let tool = FileSystemTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "metadata")
            .with_path("path", "/tmp");

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let ToolData::Structured(data) = &result.data {
            assert_eq!(data.data_type, "file_metadata");
            assert_eq!(data.fields.get("is_dir"), Some(&serde_json::json!(true)));
        } else {
            panic!("Expected structured data");
        }
    }

    #[tokio::test]
    async fn test_filesystem_permissions() {
        let tool = FileSystemTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "permissions")
            .with_path("path", "/tmp")
            .with_string("permission_type", "all");

        let result = tool.execute(&params).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_filesystem_list() {
        let tool = FileSystemTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "list")
            .with_path("path", "/tmp")
            .with_int("depth", 1);

        let result = tool.execute(&params).await;
        assert!(result.success);
        assert!(result.as_list().is_some());
    }

    #[tokio::test]
    async fn test_filesystem_is_safe() {
        let tool = FileSystemTool::new();

        // Test safe path
        let params = ToolCallParams::new()
            .with_string("operation", "is_safe")
            .with_path("path", "/tmp");
        let result = tool.execute(&params).await;
        assert!(result.success);

        // Test dangerous path
        let params = ToolCallParams::new()
            .with_string("operation", "is_safe")
            .with_path("path", "/etc/passwd");
        let result = tool.execute(&params).await;
        assert!(result.success);
        if let ToolData::Structured(data) = &result.data {
            assert_eq!(
                data.fields.get("is_system_path"),
                Some(&serde_json::json!(true))
            );
        }
    }

    #[tokio::test]
    async fn test_missing_operation() {
        let tool = FileSystemTool::new();
        let params = ToolCallParams::new().with_path("path", "/tmp");

        let result = tool.execute(&params).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("operation"));
    }
}
