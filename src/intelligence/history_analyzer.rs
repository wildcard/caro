//! Shell history analyzer
//!
//! Analyzes shell history to extract common patterns and frequently used commands.

use super::ContextError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Shell history context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryContext {
    /// Top frequently used commands
    pub frequent_commands: Vec<CommandFrequency>,
    /// Common command patterns (e.g., always uses --color)
    pub common_patterns: Vec<String>,
    /// Total commands analyzed
    pub total_commands: usize,
}

/// Command frequency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFrequency {
    /// Command name/pattern
    pub command: String,
    /// Number of occurrences
    pub count: usize,
    /// Percentage of total
    pub percentage: f64,
}

impl HistoryContext {
    /// Create empty history context
    pub fn empty() -> Self {
        Self {
            frequent_commands: Vec::new(),
            common_patterns: Vec::new(),
            total_commands: 0,
        }
    }

    /// Convert to LLM-friendly context string
    pub fn to_llm_context(&self) -> String {
        if self.frequent_commands.is_empty() {
            return "Shell History: No patterns detected".to_string();
        }

        let mut lines = Vec::new();

        lines.push(format!("Shell History ({} commands):", self.total_commands));

        // Top commands
        let top_cmds: Vec<String> = self
            .frequent_commands
            .iter()
            .take(5)
            .map(|cmd| format!("{} ({}%)", cmd.command, cmd.percentage.round() as u32))
            .collect();

        if !top_cmds.is_empty() {
            lines.push(format!("Frequent Commands: {}", top_cmds.join(", ")));
        }

        // Common patterns
        if !self.common_patterns.is_empty() {
            lines.push(format!(
                "Common Patterns: {}",
                self.common_patterns.join(", ")
            ));
        }

        lines.join("\n")
    }
}

/// Shell history analyzer
pub struct HistoryAnalyzer;

impl HistoryAnalyzer {
    /// Analyze shell history
    pub async fn analyze() -> Result<HistoryContext, ContextError> {
        let history_paths = Self::get_history_paths();

        for path in history_paths {
            if path.exists() {
                if let Ok(context) = Self::parse_history_file(&path).await {
                    return Ok(context);
                }
            }
        }

        // No history found
        Ok(HistoryContext::empty())
    }

    fn get_history_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = dirs::home_dir() {
            // Bash
            paths.push(home.join(".bash_history"));
            // Zsh
            paths.push(home.join(".zsh_history"));
            // Fish
            paths.push(home.join(".local/share/fish/fish_history"));
        }

        paths
    }

    async fn parse_history_file(path: &PathBuf) -> Result<HistoryContext, ContextError> {
        let content = fs::read_to_string(path).await?;

        // Handle different history formats
        if path.ends_with("fish_history") {
            Self::parse_fish_history(&content)
        } else if path.ends_with(".zsh_history") {
            Self::parse_zsh_history(&content)
        } else {
            Self::parse_bash_history(&content)
        }
    }

    fn parse_bash_history(content: &str) -> Result<HistoryContext, ContextError> {
        let commands: Vec<&str> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .collect();

        Self::analyze_commands(&commands)
    }

    fn parse_zsh_history(content: &str) -> Result<HistoryContext, ContextError> {
        let mut commands = Vec::new();

        for line in content.lines() {
            // Zsh history format: : timestamp:duration;command
            if let Some(cmd_part) = line.split(';').nth(1) {
                commands.push(cmd_part);
            } else if !line.starts_with(':') && !line.trim().is_empty() {
                commands.push(line);
            }
        }

        Self::analyze_commands(&commands)
    }

    fn parse_fish_history(content: &str) -> Result<HistoryContext, ContextError> {
        let mut commands = Vec::new();

        for line in content.lines() {
            // Fish history format: - cmd: command
            if line.starts_with("- cmd: ") {
                let cmd = line.trim_start_matches("- cmd: ");
                commands.push(cmd);
            }
        }

        Self::analyze_commands(&commands)
    }

    fn analyze_commands(commands: &[&str]) -> Result<HistoryContext, ContextError> {
        let total_commands = commands.len();

        if total_commands == 0 {
            return Ok(HistoryContext::empty());
        }

        // Count command frequencies
        let mut command_counts: HashMap<String, usize> = HashMap::new();

        for cmd in commands {
            // Extract base command (first word)
            let base_cmd = cmd
                .split_whitespace()
                .next()
                .unwrap_or(cmd)
                .trim()
                .to_string();

            if !base_cmd.is_empty() && !Self::is_sensitive_command(&base_cmd) {
                *command_counts.entry(base_cmd).or_insert(0) += 1;
            }
        }

        // Convert to sorted frequency list
        let mut frequent_commands: Vec<CommandFrequency> = command_counts
            .into_iter()
            .map(|(cmd, count)| CommandFrequency {
                command: cmd,
                count,
                percentage: (count as f64 / total_commands as f64) * 100.0,
            })
            .collect();

        frequent_commands.sort_by(|a, b| b.count.cmp(&a.count));
        frequent_commands.truncate(10); // Keep top 10

        // Detect common patterns
        let common_patterns = Self::detect_patterns(commands);

        Ok(HistoryContext {
            frequent_commands,
            common_patterns,
            total_commands,
        })
    }

    fn detect_patterns(commands: &[&str]) -> Vec<String> {
        let mut patterns = Vec::new();

        // Check for common flags
        let total = commands.len() as f64;
        let color_count = commands
            .iter()
            .filter(|cmd| cmd.contains("--color"))
            .count();
        let verbose_count = commands
            .iter()
            .filter(|cmd| cmd.contains("-v") || cmd.contains("--verbose"))
            .count();

        if color_count as f64 / total > 0.3 {
            patterns.push("uses --color frequently".to_string());
        }

        if verbose_count as f64 / total > 0.2 {
            patterns.push("prefers verbose output".to_string());
        }

        // Check for Git usage
        let git_count = commands.iter().filter(|cmd| cmd.starts_with("git")).count();
        if git_count as f64 / total > 0.2 {
            patterns.push("frequent Git user".to_string());
        }

        // Check for Docker usage
        let docker_count = commands
            .iter()
            .filter(|cmd| cmd.starts_with("docker"))
            .count();
        if docker_count as f64 / total > 0.1 {
            patterns.push("uses Docker regularly".to_string());
        }

        patterns
    }

    fn is_sensitive_command(cmd: &str) -> bool {
        // Filter out potentially sensitive commands
        let sensitive = [
            "password", "passwd", "secret", "token", "key", "api_key", "credential",
        ];

        sensitive.iter().any(|s| cmd.to_lowercase().contains(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_context_empty() {
        let ctx = HistoryContext::empty();
        assert!(ctx.frequent_commands.is_empty());
        assert_eq!(ctx.total_commands, 0);
    }

    #[test]
    fn test_history_context_to_llm_string() {
        let ctx = HistoryContext {
            frequent_commands: vec![
                CommandFrequency {
                    command: "git".to_string(),
                    count: 50,
                    percentage: 25.0,
                },
                CommandFrequency {
                    command: "ls".to_string(),
                    count: 30,
                    percentage: 15.0,
                },
            ],
            common_patterns: vec!["frequent Git user".to_string()],
            total_commands: 200,
        };

        let llm_str = ctx.to_llm_context();
        assert!(llm_str.contains("git"));
        assert!(llm_str.contains("Git user"));
    }

    #[test]
    fn test_parse_bash_history() {
        let content = "ls -la\ngit status\ncd /tmp\nls\n# comment\ngit commit\n";
        let result = HistoryAnalyzer::parse_bash_history(content);
        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert!(ctx.total_commands > 0);
        assert!(!ctx.frequent_commands.is_empty());
    }

    #[test]
    fn test_parse_zsh_history() {
        let content = ": 1234567890:0;ls -la\n: 1234567891:0;git status\n";
        let result = HistoryAnalyzer::parse_zsh_history(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fish_history() {
        let content = "- cmd: ls -la\n  when: 1234567890\n- cmd: git status\n";
        let result = HistoryAnalyzer::parse_fish_history(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_patterns() {
        let commands = vec![
            "git status",
            "git commit",
            "git push",
            "ls -la",
            "docker ps",
        ];
        let patterns = HistoryAnalyzer::detect_patterns(&commands);
        assert!(patterns.iter().any(|p| p.contains("Git")));
    }

    #[test]
    fn test_is_sensitive_command() {
        assert!(HistoryAnalyzer::is_sensitive_command("set-password"));
        assert!(HistoryAnalyzer::is_sensitive_command("export API_KEY=xxx"));
        assert!(!HistoryAnalyzer::is_sensitive_command("git status"));
    }

    #[tokio::test]
    async fn test_analyze_history() {
        let result = HistoryAnalyzer::analyze().await;
        assert!(result.is_ok());
        // Actual results depend on user's shell history
    }
}
