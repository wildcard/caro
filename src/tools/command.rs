//! Command Tool - Command discovery, availability, and capability inspection
//!
//! Provides context-gathering capabilities for shell commands during
//! command generation. Helps validate command availability, discover
//! flags and options, and understand command behavior on the current platform.

use super::{
    ParameterType, StructuredData, Tool, ToolCallParams, ToolCategory, ToolData, ToolParameters,
    ToolResult,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

/// Command tool for command discovery and inspection
pub struct CommandTool {
    /// Cache of command availability checks
    availability_cache: std::sync::RwLock<HashMap<String, bool>>,
    /// Maximum help text lines to return
    max_help_lines: usize,
}

impl Default for CommandTool {
    fn default() -> Self {
        Self {
            availability_cache: std::sync::RwLock::new(HashMap::new()),
            max_help_lines: 50,
        }
    }
}

impl CommandTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_help_lines(mut self, lines: usize) -> Self {
        self.max_help_lines = lines;
        self
    }

    /// Check if a command is available on the system
    fn check_available(&self, command: &str) -> ToolResult {
        let start = Instant::now();

        // Check cache first
        if let Ok(cache) = self.availability_cache.read() {
            if let Some(&available) = cache.get(command) {
                return ToolResult::success(
                    ToolData::Boolean(available),
                    start.elapsed().as_millis() as u64,
                );
            }
        }

        let available = Command::new("which")
            .arg(command)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false);

        // Update cache
        if let Ok(mut cache) = self.availability_cache.write() {
            cache.insert(command.to_string(), available);
        }

        ToolResult::success(
            ToolData::Boolean(available),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get command version
    fn get_version(&self, command: &str) -> ToolResult {
        let start = Instant::now();

        // Try common version flags
        for flag in ["--version", "-V", "-v", "version"] {
            if let Ok(output) = Command::new(command).arg(flag).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Version info might be in stdout or stderr
                let version_text = if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    stderr.to_string()
                };

                if !version_text.is_empty() {
                    // Extract first line as version
                    let first_line = version_text.lines().next().unwrap_or("").to_string();
                    return ToolResult::success(
                        ToolData::String(first_line),
                        start.elapsed().as_millis() as u64,
                    );
                }
            }
        }

        ToolResult::error(
            format!("Could not determine version for: {}", command),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get command help text
    fn get_help(&self, command: &str) -> ToolResult {
        let start = Instant::now();

        // Try common help flags
        for flag in ["--help", "-h", "-?", "help"] {
            if let Ok(output) = Command::new(command).arg(flag).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let help_text = if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    stderr.to_string()
                };

                if !help_text.is_empty() {
                    // Truncate to max lines
                    let truncated: String = help_text
                        .lines()
                        .take(self.max_help_lines)
                        .collect::<Vec<_>>()
                        .join("\n");

                    return ToolResult::success(
                        ToolData::String(truncated),
                        start.elapsed().as_millis() as u64,
                    );
                }
            }
        }

        // Try man page as fallback
        if let Ok(output) = Command::new("man").arg("-f").arg(command).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                return ToolResult::success(
                    ToolData::String(stdout.to_string()),
                    start.elapsed().as_millis() as u64,
                );
            }
        }

        ToolResult::error(
            format!("Could not get help for: {}", command),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get the full path to a command
    fn get_path(&self, command: &str) -> ToolResult {
        let start = Instant::now();

        match Command::new("which").arg(command).output() {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                ToolResult::success(ToolData::String(path), start.elapsed().as_millis() as u64)
            }
            _ => ToolResult::error(
                format!("Command not found: {}", command),
                start.elapsed().as_millis() as u64,
            ),
        }
    }

    /// Get comprehensive command information
    fn get_info(&self, command: &str) -> ToolResult {
        let start = Instant::now();

        let available = Command::new("which")
            .arg(command)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false);

        if !available {
            return ToolResult::error(
                format!("Command not found: {}", command),
                start.elapsed().as_millis() as u64,
            );
        }

        let path = Command::new("which")
            .arg(command)
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string());

        let version = Command::new(command)
            .arg("--version")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|s| s.lines().next().map(|l| l.to_string()));

        let command_type = self.detect_command_type(command, path.as_deref());

        let mut data = StructuredData::new("command_info");
        data = data
            .with_field("name", command)
            .with_field("available", true)
            .with_field("command_type", command_type);

        if let Some(p) = path {
            data = data.with_field("path", p);
        }
        if let Some(v) = version {
            data = data.with_field("version", v);
        }

        ToolResult::success(
            ToolData::Structured(data),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Detect command type (GNU, BSD, busybox, etc.)
    fn detect_command_type(&self, command: &str, path: Option<&str>) -> String {
        // Check if it's a busybox applet
        if let Some(p) = path {
            if p.contains("busybox") {
                return "busybox".to_string();
            }
        }

        // Try to detect from version output
        if let Ok(output) = Command::new(command).arg("--version").output() {
            let version = String::from_utf8_lossy(&output.stdout).to_lowercase();

            if version.contains("gnu") {
                return "gnu".to_string();
            } else if version.contains("bsd") {
                return "bsd".to_string();
            } else if version.contains("busybox") {
                return "busybox".to_string();
            }
        }

        // Default based on OS
        #[cfg(target_os = "macos")]
        {
            "bsd".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "gnu".to_string()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            "unknown".to_string()
        }
    }

    /// Check multiple commands for availability
    fn check_batch_available(&self, commands: &[String]) -> ToolResult {
        let start = Instant::now();

        let mut results: HashMap<String, String> = HashMap::new();

        for cmd in commands {
            let available = Command::new("which")
                .arg(cmd)
                .output()
                .map(|out| out.status.success())
                .unwrap_or(false);

            results.insert(cmd.clone(), available.to_string());
        }

        ToolResult::success(ToolData::Map(results), start.elapsed().as_millis() as u64)
    }

    /// Find alternatives for a command
    fn find_alternatives(&self, command: &str) -> ToolResult {
        let start = Instant::now();

        let alternatives: &[(&str, &[&str])] = &[
            ("ss", &["netstat", "lsof"]),
            ("netstat", &["ss", "lsof"]),
            ("vim", &["nvim", "vi", "nano"]),
            ("nvim", &["vim", "vi", "nano"]),
            ("rg", &["grep", "ag"]),
            ("fd", &["find"]),
            ("bat", &["cat", "less"]),
            ("exa", &["ls", "eza"]),
            ("eza", &["ls", "exa"]),
            ("htop", &["top", "btop"]),
            ("dua", &["du", "ncdu"]),
            ("procs", &["ps"]),
            ("sed", &["gsed"]),
            ("awk", &["gawk"]),
            ("grep", &["ggrep", "rg"]),
            ("find", &["gfind", "fd"]),
            ("readlink", &["greadlink"]),
            ("stat", &["gstat"]),
        ];

        let alts = alternatives
            .iter()
            .find(|(cmd, _)| *cmd == command)
            .map(|(_, alts)| *alts)
            .unwrap_or(&[]);

        let available_alts: Vec<String> = alts
            .iter()
            .filter(|alt| {
                Command::new("which")
                    .arg(alt)
                    .output()
                    .map(|out| out.status.success())
                    .unwrap_or(false)
            })
            .map(|s| s.to_string())
            .collect();

        ToolResult::success(
            ToolData::List(available_alts),
            start.elapsed().as_millis() as u64,
        )
    }

    /// Get platform-specific flags for a command
    fn get_platform_flags(&self, command: &str) -> ToolResult {
        let start = Instant::now();

        let command_type = self.detect_command_type(command, None);

        let platform_flags = match (command, command_type.as_str()) {
            ("ps", "bsd") => StructuredData::new("platform_flags")
                .with_field("sort_by_cpu", "ps aux | sort -nrk 3")
                .with_field("sort_by_mem", "ps aux | sort -nrk 4")
                .with_field("note", "BSD ps doesn't support --sort flag"),
            ("ps", "gnu") => StructuredData::new("platform_flags")
                .with_field("sort_by_cpu", "ps aux --sort=-pcpu")
                .with_field("sort_by_mem", "ps aux --sort=-pmem")
                .with_field("note", "GNU ps supports --sort flag"),
            ("sed", "bsd") => StructuredData::new("platform_flags")
                .with_field("in_place", "sed -i ''")
                .with_field("note", "BSD sed requires empty string after -i"),
            ("sed", "gnu") => StructuredData::new("platform_flags")
                .with_field("in_place", "sed -i")
                .with_field("note", "GNU sed uses -i without argument"),
            ("du", "bsd") => StructuredData::new("platform_flags")
                .with_field("depth", "-d")
                .with_field("example", "du -d 1 -h")
                .with_field("note", "BSD uses -d for depth"),
            ("du", "gnu") => StructuredData::new("platform_flags")
                .with_field("depth", "--max-depth")
                .with_field("example", "du --max-depth=1 -h")
                .with_field("note", "GNU uses --max-depth"),
            ("date", "bsd") => StructuredData::new("platform_flags")
                .with_field("relative", "-v")
                .with_field("example", "date -v-7d")
                .with_field("note", "BSD uses -v for relative dates"),
            ("date", "gnu") => StructuredData::new("platform_flags")
                .with_field("relative", "--date")
                .with_field("example", "date --date='7 days ago'")
                .with_field("note", "GNU uses --date for relative dates"),
            _ => StructuredData::new("platform_flags")
                .with_field("command", command)
                .with_field("type", command_type.clone())
                .with_field("note", "No platform-specific flags documented"),
        };

        ToolResult::success(
            ToolData::Structured(platform_flags),
            start.elapsed().as_millis() as u64,
        )
    }
}

#[async_trait]
impl Tool for CommandTool {
    fn name(&self) -> &str {
        "command"
    }

    fn description(&self) -> &str {
        "Command discovery: check availability, version, help, paths, and platform-specific flags"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Command
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new()
            .with_required("operation", ParameterType::String, "Operation: available, version, help, path, info, batch_available, alternatives, platform_flags")
            .with_required("command", ParameterType::String, "The command to inspect")
            .with_optional("commands", ParameterType::StringArray, "Multiple commands for batch operations")
    }

    async fn execute(&self, params: &ToolCallParams) -> ToolResult {
        let start = Instant::now();

        let operation = match params.get_string("operation") {
            Some(op) => op,
            None => return ToolResult::error("Missing required parameter: operation", 0),
        };

        let command = params.get_string("command").unwrap_or("");

        match operation {
            "available" => {
                if command.is_empty() {
                    return ToolResult::error("Missing required parameter: command", 0);
                }
                self.check_available(command)
            }
            "version" => {
                if command.is_empty() {
                    return ToolResult::error("Missing required parameter: command", 0);
                }
                self.get_version(command)
            }
            "help" => {
                if command.is_empty() {
                    return ToolResult::error("Missing required parameter: command", 0);
                }
                self.get_help(command)
            }
            "path" => {
                if command.is_empty() {
                    return ToolResult::error("Missing required parameter: command", 0);
                }
                self.get_path(command)
            }
            "info" => {
                if command.is_empty() {
                    return ToolResult::error("Missing required parameter: command", 0);
                }
                self.get_info(command)
            }
            "batch_available" => {
                let commands = params.get_string_array("commands");
                match commands {
                    Some(cmds) if !cmds.is_empty() => self.check_batch_available(cmds),
                    _ => ToolResult::error(
                        "Missing required parameter: commands",
                        start.elapsed().as_millis() as u64,
                    ),
                }
            }
            "alternatives" => {
                if command.is_empty() {
                    return ToolResult::error("Missing required parameter: command", 0);
                }
                self.find_alternatives(command)
            }
            "platform_flags" => {
                if command.is_empty() {
                    return ToolResult::error("Missing required parameter: command", 0);
                }
                self.get_platform_flags(command)
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
    async fn test_command_available() {
        let tool = CommandTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "available")
            .with_string("command", "ls");

        let result = tool.execute(&params).await;
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[tokio::test]
    async fn test_command_not_available() {
        let tool = CommandTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "available")
            .with_string("command", "nonexistent_command_xyz");

        let result = tool.execute(&params).await;
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[tokio::test]
    async fn test_command_path() {
        let tool = CommandTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "path")
            .with_string("command", "ls");

        let result = tool.execute(&params).await;
        assert!(result.success);
        assert!(result.as_string().is_some());
    }

    #[tokio::test]
    async fn test_command_info() {
        let tool = CommandTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "info")
            .with_string("command", "ls");

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let ToolData::Structured(data) = &result.data {
            assert_eq!(data.data_type, "command_info");
            assert!(data.fields.contains_key("available"));
        }
    }

    #[tokio::test]
    async fn test_batch_available() {
        let tool = CommandTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "batch_available")
            .with_string_array(
                "commands",
                vec![
                    "ls".to_string(),
                    "cat".to_string(),
                    "nonexistent_xyz".to_string(),
                ],
            );

        let result = tool.execute(&params).await;
        assert!(result.success);

        if let Some(map) = result.as_map() {
            assert_eq!(map.get("ls"), Some(&"true".to_string()));
            assert_eq!(map.get("nonexistent_xyz"), Some(&"false".to_string()));
        }
    }

    #[tokio::test]
    async fn test_platform_flags() {
        let tool = CommandTool::new();
        let params = ToolCallParams::new()
            .with_string("operation", "platform_flags")
            .with_string("command", "ps");

        let result = tool.execute(&params).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_missing_operation() {
        let tool = CommandTool::new();
        let params = ToolCallParams::new().with_string("command", "ls");

        let result = tool.execute(&params).await;
        assert!(!result.success);
    }
}
