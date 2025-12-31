//! Context Tool - System and environment context gathering
//!
//! Provides context-gathering capabilities for system information,
//! environment variables, shell capabilities, and platform detection
//! during command generation.

use super::{
    ParameterType, StructuredData, Tool, ToolCallParams, ToolCategory, ToolData, ToolParameters,
    ToolResult,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

/// Context tool for gathering system and environment information
pub struct ContextTool {
    /// Sensitive environment variable patterns to filter
    sensitive_patterns: Vec<String>,
}

impl Default for ContextTool {
    fn default() -> Self {
        Self {
            sensitive_patterns: vec![
                "KEY".to_string(),
                "SECRET".to_string(),
                "TOKEN".to_string(),
                "PASSWORD".to_string(),
                "CREDENTIAL".to_string(),
                "API".to_string(),
                "AUTH".to_string(),
                "PRIVATE".to_string(),
            ],
        }
    }
}

impl ContextTool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an environment variable name is sensitive
    fn is_sensitive(&self, name: &str) -> bool {
        let upper = name.to_uppercase();
        self.sensitive_patterns
            .iter()
            .any(|pat| upper.contains(pat))
    }

    /// Get operating system information
    fn get_os_info(&self) -> ToolResult {
        let start = Instant::now();

        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let mut data = StructuredData::new("os_info")
            .with_field("os", os)
            .with_field("arch", arch);

        // Get OS-specific version info
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                data = data.with_field("version", version);
            }
            if let Ok(output) = Command::new("sw_vers").arg("-productName").output() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                data = data.with_field("name", name);
            }
            data = data.with_field("command_style", "bsd");
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("uname").arg("-r").output() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                data = data.with_field("kernel_version", version);
            }
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        let name = line
                            .trim_start_matches("PRETTY_NAME=")
                            .trim_matches('"')
                            .to_string();
                        data = data.with_field("distribution", name);
                        break;
                    }
                }
            }
            data = data.with_field("command_style", "gnu");
        }

        ToolResult::success(
            ToolData::Structured(data),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get shell information
    fn get_shell_info(&self) -> ToolResult {
        let start = Instant::now();

        let shell_path = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell_name = shell_path.rsplit('/').next().unwrap_or("sh").to_string();

        let mut data = StructuredData::new("shell_info")
            .with_field("path", shell_path)
            .with_field("name", shell_name.clone());

        // Get shell version
        let version_flag = match shell_name.as_str() {
            "zsh" => Some("--version"),
            "bash" => Some("--version"),
            "fish" => Some("--version"),
            _ => None,
        };

        if let Some(flag) = version_flag {
            if let Ok(output) = Command::new(&shell_name).arg(flag).output() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                data = data.with_field("version", version);
            }
        }

        // Add shell capabilities
        let capabilities = match shell_name.as_str() {
            "zsh" => vec![
                "arrays",
                "associative_arrays",
                "completion",
                "globbing",
                "history",
            ],
            "bash" => vec!["arrays", "completion", "history", "job_control"],
            "fish" => vec![
                "arrays",
                "autosuggestions",
                "completion",
                "syntax_highlighting",
            ],
            _ => vec!["posix_compliant"],
        };

        data = data.with_field("capabilities", serde_json::json!(capabilities));

        ToolResult::success(
            ToolData::Structured(data),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get current working directory info
    fn get_cwd_info(&self) -> ToolResult {
        let start = Instant::now();

        match std::env::current_dir() {
            Ok(cwd) => {
                let cwd_str = cwd.to_string_lossy().to_string();

                let mut data = StructuredData::new("cwd_info").with_field("path", cwd_str.clone());

                // Check if it's a git repository
                let is_git = cwd.join(".git").exists();
                data = data.with_field("is_git_repo", is_git);

                // Get git branch if applicable
                if is_git {
                    if let Ok(output) = Command::new("git")
                        .args(["rev-parse", "--abbrev-ref", "HEAD"])
                        .output()
                    {
                        if output.status.success() {
                            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            data = data.with_field("git_branch", branch);
                        }
                    }
                }

                // Check for common project files
                let project_markers = [
                    ("Cargo.toml", "rust"),
                    ("package.json", "node"),
                    ("pyproject.toml", "python"),
                    ("go.mod", "go"),
                    ("Makefile", "make"),
                    ("CMakeLists.txt", "cmake"),
                ];

                for (file, project_type) in project_markers {
                    if cwd.join(file).exists() {
                        data = data.with_field("project_type", project_type);
                        break;
                    }
                }

                ToolResult::success(
                    ToolData::Structured(data),
                    start.elapsed().as_millis() as u64,
                )
            }
            Err(e) => ToolResult::error(
                format!("Failed to get current directory: {}", e),
                start.elapsed().as_millis() as u64,
            ),
        }
    }

    /// Get environment variable (filtered for sensitive data)
    fn get_env_var(&self, name: &str) -> ToolResult {
        let start = Instant::now();

        if self.is_sensitive(name) {
            return ToolResult::error(
                format!("Access to sensitive variable '{}' is restricted", name),
                start.elapsed().as_millis() as u64,
            );
        }

        match std::env::var(name) {
            Ok(value) => {
                ToolResult::success(ToolData::String(value), start.elapsed().as_millis() as u64)
            }
            Err(_) => ToolResult::error(
                format!("Environment variable '{}' not set", name),
                start.elapsed().as_millis() as u64,
            ),
        }
    }

    /// List non-sensitive environment variables
    fn list_env_vars(&self, filter: Option<&str>) -> ToolResult {
        let start = Instant::now();

        let mut vars: HashMap<String, String> = HashMap::new();

        for (key, value) in std::env::vars() {
            // Skip sensitive variables
            if self.is_sensitive(&key) {
                continue;
            }

            // Apply filter if specified
            if let Some(f) = filter {
                if !key.to_lowercase().contains(&f.to_lowercase()) {
                    continue;
                }
            }

            // Truncate long values
            let truncated_value = if value.len() > 100 {
                format!("{}...", &value[..100])
            } else {
                value
            };

            vars.insert(key, truncated_value);
        }

        ToolResult::success(ToolData::Map(vars), start.elapsed().as_millis() as u64)
    }

    /// Get user information
    fn get_user_info(&self) -> ToolResult {
        let start = Instant::now();

        let username = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());

        let mut data = StructuredData::new("user_info")
            .with_field("username", username.clone())
            .with_field("home", home);

        // Check if running as root
        let is_root = username == "root"
            || std::env::var("EUID")
                .ok()
                .and_then(|e| e.parse::<u32>().ok())
                == Some(0);

        data = data.with_field("is_root", is_root);

        // Get groups if available
        #[cfg(unix)]
        {
            if let Ok(output) = Command::new("groups").output() {
                let groups = String::from_utf8_lossy(&output.stdout).trim().to_string();
                data = data.with_field("groups", groups);
            }
        }

        ToolResult::success(
            ToolData::Structured(data),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get comprehensive system context
    fn get_full_context(&self) -> ToolResult {
        let start = Instant::now();

        let os_info = self.get_os_info();
        let shell_info = self.get_shell_info();
        let cwd_info = self.get_cwd_info();
        let user_info = self.get_user_info();

        let mut context = StructuredData::new("full_context");

        // Extract data from each sub-result
        if let ToolData::Structured(os) = os_info.data {
            context = context.with_field("os", serde_json::to_value(os.fields).unwrap());
        }
        if let ToolData::Structured(shell) = shell_info.data {
            context = context.with_field("shell", serde_json::to_value(shell.fields).unwrap());
        }
        if let ToolData::Structured(cwd) = cwd_info.data {
            context = context.with_field("cwd", serde_json::to_value(cwd.fields).unwrap());
        }
        if let ToolData::Structured(user) = user_info.data {
            context = context.with_field("user", serde_json::to_value(user.fields).unwrap());
        }

        ToolResult::success(
            ToolData::Structured(context),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Check if a path is in PATH
    fn check_path_contains(&self, dir: &str) -> ToolResult {
        let start = Instant::now();

        let path = std::env::var("PATH").unwrap_or_default();
        let contains = path.split(':').any(|p| p == dir);

        ToolResult::success(
            ToolData::Boolean(contains),
            start.elapsed().as_millis() as u64,
        )
    }
}

#[async_trait]
impl Tool for ContextTool {
    fn name(&self) -> &str {
        "context"
    }

    fn description(&self) -> &str {
        "System context: OS info, shell, environment, user, working directory"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Context
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .with_required(
                "operation",
                ParameterType::String,
                "Operation: os, shell, cwd, env, list_env, user, full, path_contains",
            )
            .with_optional(
                "name",
                ParameterType::String,
                "Variable name for env operation",
            )
            .with_optional(
                "filter",
                ParameterType::String,
                "Filter pattern for list_env",
            )
            .with_optional("dir", ParameterType::String, "Directory for path_contains")
    }

    async fn execute(&self, params: &ToolCallParams) -> ToolResult {
        let start = Instant::now();

        let operation = match params.get_string("operation") {
            Some(op) => op,
            None => return ToolResult::error("Missing required parameter: operation", 0),
        };

        match operation {
            "os" => self.get_os_info(),
            "shell" => self.get_shell_info(),
            "cwd" => self.get_cwd_info(),
            "env" => {
                let name = params.get_string("name").unwrap_or("");
                if name.is_empty() {
                    return ToolResult::error("Missing required parameter: name", 0);
                }
                self.get_env_var(name)
            }
            "list_env" => {
                let filter = params.get_string("filter");
                self.list_env_vars(filter)
            }
            "user" => self.get_user_info(),
            "full" => self.get_full_context(),
            "path_contains" => {
                let dir = params.get_string("dir").unwrap_or("");
                if dir.is_empty() {
                    return ToolResult::error("Missing required parameter: dir", 0);
                }
                self.check_path_contains(dir)
            }
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
    async fn test_os_info() {
        let tool = ContextTool::new();
        let params = ToolCallParams::new().with_string("operation", "os");

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let ToolData::Structured(data) = &result.data {
            assert_eq!(data.data_type, "os_info");
            assert!(data.fields.contains_key("os"));
            assert!(data.fields.contains_key("arch"));
        }
    }

    #[tokio::test]
    async fn test_shell_info() {
        let tool = ContextTool::new();
        let params = ToolCallParams::new().with_string("operation", "shell");

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let ToolData::Structured(data) = &result.data {
            assert_eq!(data.data_type, "shell_info");
        }
    }

    #[tokio::test]
    async fn test_cwd_info() {
        let tool = ContextTool::new();
        let params = ToolCallParams::new().with_string("operation", "cwd");

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let ToolData::Structured(data) = &result.data {
            assert_eq!(data.data_type, "cwd_info");
            assert!(data.fields.contains_key("path"));
        }
    }

    #[tokio::test]
    async fn test_env_var() {
        let tool = ContextTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "env")
            .with_string("name", "PATH");

        let result = tool.execute(&params).await;
        assert!(result.success);
        assert!(result.as_string().is_some());
    }

    #[tokio::test]
    async fn test_sensitive_env_blocked() {
        let tool = ContextTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "env")
            .with_string("name", "MY_API_KEY");

        let result = tool.execute(&params).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("restricted"));
    }

    #[tokio::test]
    async fn test_user_info() {
        let tool = ContextTool::new();
        let params = ToolCallParams::new().with_string("operation", "user");

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let ToolData::Structured(data) = &result.data {
            assert!(data.fields.contains_key("username"));
        }
    }

    #[tokio::test]
    async fn test_full_context() {
        let tool = ContextTool::new();
        let params = ToolCallParams::new().with_string("operation", "full");

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let ToolData::Structured(data) = &result.data {
            assert_eq!(data.data_type, "full_context");
        }
    }
}
