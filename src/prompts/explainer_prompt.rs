//! Explainer Prompt System
//!
//! This module provides a prompt system optimized for educational command explanations.
//! Unlike the generator prompt which focuses on quick command synthesis, this system
//! provides detailed explanations of commands, options, and alternatives.
//!
//! # Design Philosophy
//!
//! The explainer prompt is inspired by how expert developers explain commands:
//!
//! 1. **Identify the Tool**: Recognize the relevant Unix utility for the task
//! 2. **Explain the Command**: Break down what the command does
//! 3. **Describe Options**: Explain each flag and option used
//! 4. **Show Examples**: Provide variations for different use cases
//!
//! # Example Output
//!
//! For "find files modified in the last 24 hours":
//!
//! ```text
//! Use `find` with time-based filters:
//!
//!   # Last 24 hours
//!   find . -type f -mtime -1
//!
//!   # With file details
//!   find . -type f -mtime -1 -ls
//!
//! The `-mtime -1` means "modified less than 1 day ago".
//! Use `-mmin` for minute precision (1440 minutes = 24 hours).
//! ```

use super::capability_profile::{CapabilityProfile, ProfileType};
use super::profiles::{
    AlternativeCommand, CommandExplanation, OptionExplanation, ProfileConfig, UsageExample,
};

/// Builder for explainer-mode prompts
///
/// This prompt builder generates responses that explain commands in detail,
/// suitable for users who want to learn how shell commands work.
pub struct ExplainerPromptBuilder {
    profile: CapabilityProfile,
    #[allow(dead_code)]
    config: ProfileConfig,
    #[allow(dead_code)]
    current_directory: Option<String>,
    #[allow(dead_code)]
    available_context: Option<String>,
}

impl ExplainerPromptBuilder {
    /// Create a new explainer prompt builder with the given capability profile
    pub fn new(profile: CapabilityProfile) -> Self {
        Self {
            profile,
            config: ProfileConfig::explainer(),
            current_directory: None,
            available_context: None,
        }
    }

    /// Create with custom profile config
    pub fn with_config(profile: CapabilityProfile, config: ProfileConfig) -> Self {
        Self {
            profile,
            config,
            current_directory: None,
            available_context: None,
        }
    }

    /// Create an Ubuntu-optimized explainer prompt builder
    pub fn ubuntu() -> Self {
        Self::new(CapabilityProfile::ubuntu())
    }

    /// Set current working directory context
    pub fn current_directory(mut self, dir: impl Into<String>) -> Self {
        self.current_directory = Some(dir.into());
        self
    }

    /// Set additional context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.available_context = Some(context.into());
        self
    }

    /// Build the system prompt for explanation mode
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = String::new();

        // Role definition
        prompt.push_str(&self.build_role_section());
        prompt.push('\n');

        // Output format for explanations
        prompt.push_str(&self.build_output_format());
        prompt.push('\n');

        // Tool knowledge base
        prompt.push_str(&self.build_tool_knowledge());
        prompt.push('\n');

        // Platform context
        prompt.push_str(&self.build_platform_context());
        prompt.push('\n');

        // Few-shot examples
        prompt.push_str(&self.build_examples());
        prompt.push('\n');

        prompt.push_str("\nRespond to the next user message using the format above.\n");

        prompt
    }

    /// Format a complete chat prompt with system and user messages
    pub fn format_chat(&self, user_query: &str) -> String {
        let system_prompt = self.build_system_prompt();

        format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            system_prompt, user_query
        )
    }

    fn build_role_section(&self) -> String {
        r#"You are ShellCommandExplainer, an expert Unix systems educator.

GOAL: Given a user's question about shell commands, provide:
1. The BEST command for their task
2. A CLEAR explanation of how it works
3. Examples showing common variations

Your explanations should be concise yet educational, like a senior engineer helping a colleague."#
            .to_string()
    }

    fn build_output_format(&self) -> String {
        r#"OUTPUT FORMAT:

Start with a brief intro identifying the relevant tool, then show commands with comments:

```
Use `<tool>` with <key concept>:

  # <description>
  <command>

  # <variation description>
  <command variation>
```

After commands, add a brief explanation of key options.

RULES:
1. Start response with "Use `<tool>`..." identifying the main Unix utility
2. Show 2-4 command examples with # comments explaining each
3. End with 1-2 sentences explaining the most important options
4. Keep total response under 15 lines
5. Focus on POSIX-compatible commands when possible"#
            .to_string()
    }

    fn build_tool_knowledge(&self) -> String {
        let mut section = String::from("[TOOL KNOWLEDGE]\n");
        section.push_str("Common tasks and their primary tools:\n\n");

        section.push_str("FINDING FILES:\n");
        section.push_str("- find: locate files by name, type, size, date, permissions\n");
        section.push_str("  Key options: -name, -type f/d, -size +/-N, -mtime +/-N, -exec\n\n");

        section.push_str("SEARCHING TEXT:\n");
        section.push_str("- grep: search file contents for patterns\n");
        section.push_str("  Key options: -r (recursive), -i (ignore case), -n (line numbers), -l (files only)\n\n");

        section.push_str("LISTING & SORTING:\n");
        section.push_str("- ls: list directory contents\n");
        section.push_str("  Key options: -l (long), -a (all), -h (human sizes), -t (by time), -S (by size)\n");
        section.push_str("- sort: sort lines of text\n");
        section.push_str("  Key options: -n (numeric), -r (reverse), -k N (by column N), -h (human sizes)\n\n");

        section.push_str("DISK & SIZE:\n");
        section.push_str("- du: disk usage by directory\n");
        section.push_str("  Key options: -s (summary), -h (human), -d N (depth)\n");
        section.push_str("- df: disk space on filesystems\n");
        section.push_str("  Key options: -h (human readable)\n\n");

        section.push_str("TEXT PROCESSING:\n");
        section.push_str("- awk: field-based text processing\n");
        section.push_str("  Pattern: awk '{print $1}' (print first field)\n");
        section.push_str("- sed: stream editing/substitution\n");
        section.push_str("  Pattern: sed 's/old/new/g' (global substitution)\n");
        section.push_str("- cut: extract columns\n");
        section.push_str("  Pattern: cut -d',' -f1 (first comma-separated field)\n\n");

        section.push_str("PROCESSES:\n");
        section.push_str("- ps: list processes\n");
        section.push_str("  Pattern: ps aux (all processes, detailed)\n");
        section.push_str("- top/htop: interactive process viewer\n\n");

        section.push_str("NETWORK:\n");
        section.push_str("- netstat/ss: network connections and ports\n");
        section.push_str("- lsof: list open files (including network)\n");
        section.push_str("  Pattern: lsof -i :PORT (find process on port)\n");

        section
    }

    fn build_platform_context(&self) -> String {
        let mut section = format!(
            "[PLATFORM]\nOS: {} {}\nProfile: {}\n",
            self.profile.os_name, self.profile.os_version, self.profile.profile_type
        );

        // Add platform-specific notes
        match self.profile.profile_type {
            ProfileType::Bsd => {
                section.push_str("\nBSD/macOS notes:\n");
                section.push_str("- Use `stat -f '%z %N'` instead of `stat -c '%s %n'`\n");
                section.push_str("- find doesn't support -printf, use -exec stat instead\n");
                section.push_str("- sed -i requires '' argument: sed -i '' 's/...'\n");
            }
            ProfileType::GnuLinux => {
                section.push_str("\nGNU/Linux notes:\n");
                section.push_str("- Full GNU coreutils available\n");
                section.push_str("- find supports -printf for custom output\n");
                section.push_str("- grep supports -P for Perl regex\n");
            }
            ProfileType::Busybox => {
                section.push_str("\nBusyBox notes:\n");
                section.push_str("- Limited flag support, prefer POSIX options\n");
                section.push_str("- Some GNU extensions unavailable\n");
            }
            _ => {}
        }

        if let Some(ctx) = &self.available_context {
            section.push_str(&format!("\n[CONTEXT]\n{}\n", ctx));
        }

        section
    }

    fn build_examples(&self) -> String {
        let mut section = String::from("[EXAMPLES]\n\n");

        // Example 1: Finding files
        section.push_str("User: \"how to find files modified in last 24 hours\"\n");
        section.push_str("Assistant: Use `find` with time-based filters:\n\n");
        section.push_str("  # Last 24 hours\n");
        section.push_str("  find . -type f -mtime -1\n\n");
        section.push_str("  # Last 24 hours with details\n");
        section.push_str("  find . -type f -mtime -1 -ls\n\n");
        section.push_str("  # Specific directory, last 24 hours\n");
        section.push_str("  find /path/to/dir -type f -mtime -1\n\n");
        section.push_str("  # Last N minutes (more precise)\n");
        section.push_str("  find . -type f -mmin -1440  # 1440 minutes = 24 hours\n\n");
        section.push_str(
            "The `-mtime -1` means \"modified less than 1 day ago\". Use `-mmin` for minute precision.\n\n",
        );

        // Example 2: Disk usage
        section.push_str("User: \"show disk usage by folder sorted by size\"\n");
        section.push_str("Assistant: Use `du` with `sort` for size-ordered output:\n\n");
        section.push_str("  # Top 10 largest directories\n");
        section.push_str("  du -sh */ | sort -rh | head -10\n\n");
        section.push_str("  # All subdirectories with human-readable sizes\n");
        section.push_str("  du -h --max-depth=1 | sort -rh\n\n");
        section.push_str("  # Specific path\n");
        section.push_str("  du -sh /var/log/* | sort -rh\n\n");
        section.push_str(
            "The `-s` flag summarizes each argument, `-h` gives human-readable sizes, and `sort -rh` sorts largest first.\n\n",
        );

        // Example 3: Searching text
        section.push_str("User: \"search for TODO in all python files\"\n");
        section.push_str("Assistant: Use `grep` for text search across files:\n\n");
        section.push_str("  # Basic recursive search\n");
        section.push_str("  grep -rn 'TODO' --include='*.py' .\n\n");
        section.push_str("  # Case-insensitive\n");
        section.push_str("  grep -rni 'todo' --include='*.py' .\n\n");
        section.push_str("  # Just show matching files\n");
        section.push_str("  grep -rl 'TODO' --include='*.py' .\n\n");
        section.push_str(
            "The `-r` flag searches recursively, `-n` shows line numbers, and `--include` filters by filename pattern.\n",
        );

        section
    }

    /// Generate a structured explanation for a command (for programmatic use)
    pub fn create_explanation(
        &self,
        command: &str,
        intent: &str,
    ) -> CommandExplanation {
        // This would typically be generated by the LLM, but we can provide
        // static explanations for common patterns
        let tool = self.identify_primary_tool(command);

        CommandExplanation {
            command: command.to_string(),
            summary: format!("Uses {} to {}", tool, intent),
            detailed_explanation: self.generate_explanation_for_command(command, &tool),
            option_breakdown: self.extract_options(command, &tool),
            examples: self.generate_examples(&tool, intent),
            alternatives: self.generate_alternatives(&tool),
            tool_used: tool,
            use_cases: vec![intent.to_string()],
        }
    }

    fn identify_primary_tool(&self, command: &str) -> String {
        let command = command.trim();
        // Get first word (the command name)
        command
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    fn generate_explanation_for_command(&self, command: &str, tool: &str) -> String {
        match tool {
            "find" => format!(
                "The `find` command searches for files in a directory hierarchy. \
                 This command ({}) searches from the specified starting point and \
                 applies the given filters to locate matching files.",
                command
            ),
            "grep" => format!(
                "The `grep` command searches for text patterns within files. \
                 This command ({}) searches for the specified pattern in the given files or directories.",
                command
            ),
            "du" => format!(
                "The `du` command estimates file space usage. \
                 This command ({}) calculates and displays disk usage for the specified paths.",
                command
            ),
            "ls" => format!(
                "The `ls` command lists directory contents. \
                 This command ({}) displays files and directories with the specified options.",
                command
            ),
            _ => format!(
                "This command ({}) uses the `{}` utility to perform the requested operation.",
                command, tool
            ),
        }
    }

    fn extract_options(&self, command: &str, tool: &str) -> Vec<OptionExplanation> {
        let mut options = Vec::new();

        // Parse common options based on tool
        match tool {
            "find" => {
                if command.contains("-type f") {
                    options.push(OptionExplanation {
                        option: "-type f".to_string(),
                        description: "Match only regular files (not directories)".to_string(),
                        example_value: None,
                    });
                }
                if command.contains("-type d") {
                    options.push(OptionExplanation {
                        option: "-type d".to_string(),
                        description: "Match only directories".to_string(),
                        example_value: None,
                    });
                }
                if command.contains("-name") {
                    options.push(OptionExplanation {
                        option: "-name".to_string(),
                        description: "Match files by name pattern (case-sensitive)".to_string(),
                        example_value: Some("'*.txt'".to_string()),
                    });
                }
                if command.contains("-mtime") {
                    options.push(OptionExplanation {
                        option: "-mtime".to_string(),
                        description: "Match by modification time in days (+N older, -N newer)"
                            .to_string(),
                        example_value: Some("-1 (last 24 hours)".to_string()),
                    });
                }
                if command.contains("-size") {
                    options.push(OptionExplanation {
                        option: "-size".to_string(),
                        description: "Match by file size (+N larger, -N smaller)".to_string(),
                        example_value: Some("+100M (larger than 100MB)".to_string()),
                    });
                }
            }
            "grep" => {
                if command.contains("-r") || command.contains("-R") {
                    options.push(OptionExplanation {
                        option: "-r/-R".to_string(),
                        description: "Search recursively through directories".to_string(),
                        example_value: None,
                    });
                }
                if command.contains("-n") {
                    options.push(OptionExplanation {
                        option: "-n".to_string(),
                        description: "Show line numbers with output".to_string(),
                        example_value: None,
                    });
                }
                if command.contains("-i") {
                    options.push(OptionExplanation {
                        option: "-i".to_string(),
                        description: "Case-insensitive matching".to_string(),
                        example_value: None,
                    });
                }
                if command.contains("-l") {
                    options.push(OptionExplanation {
                        option: "-l".to_string(),
                        description: "Only print names of matching files".to_string(),
                        example_value: None,
                    });
                }
            }
            "ls" => {
                if command.contains("-l") {
                    options.push(OptionExplanation {
                        option: "-l".to_string(),
                        description: "Long listing format with details".to_string(),
                        example_value: None,
                    });
                }
                if command.contains("-a") {
                    options.push(OptionExplanation {
                        option: "-a".to_string(),
                        description: "Show hidden files (starting with .)".to_string(),
                        example_value: None,
                    });
                }
                if command.contains("-h") {
                    options.push(OptionExplanation {
                        option: "-h".to_string(),
                        description: "Human-readable file sizes".to_string(),
                        example_value: None,
                    });
                }
            }
            _ => {}
        }

        options
    }

    fn generate_examples(&self, tool: &str, _intent: &str) -> Vec<UsageExample> {
        match tool {
            "find" => vec![
                UsageExample {
                    description: "Find all Python files".to_string(),
                    command: "find . -name '*.py' -type f".to_string(),
                },
                UsageExample {
                    description: "Find files larger than 100MB".to_string(),
                    command: "find . -type f -size +100M".to_string(),
                },
                UsageExample {
                    description: "Find and delete empty directories".to_string(),
                    command: "find . -type d -empty -delete".to_string(),
                },
            ],
            "grep" => vec![
                UsageExample {
                    description: "Search recursively with line numbers".to_string(),
                    command: "grep -rn 'pattern' .".to_string(),
                },
                UsageExample {
                    description: "Case-insensitive search".to_string(),
                    command: "grep -ri 'pattern' .".to_string(),
                },
                UsageExample {
                    description: "Search in specific file types".to_string(),
                    command: "grep -rn 'pattern' --include='*.js' .".to_string(),
                },
            ],
            _ => vec![],
        }
    }

    fn generate_alternatives(&self, tool: &str) -> Vec<AlternativeCommand> {
        match tool {
            "find" => vec![
                AlternativeCommand {
                    command: "fd".to_string(),
                    reason: "Faster, simpler syntax (requires installation)".to_string(),
                },
                AlternativeCommand {
                    command: "locate".to_string(),
                    reason: "Faster for filename search (uses database, may be stale)".to_string(),
                },
            ],
            "grep" => vec![
                AlternativeCommand {
                    command: "rg (ripgrep)".to_string(),
                    reason: "Much faster, respects .gitignore (requires installation)".to_string(),
                },
                AlternativeCommand {
                    command: "ag (silver searcher)".to_string(),
                    reason: "Faster than grep, good defaults (requires installation)".to_string(),
                },
            ],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explainer_prompt_builder() {
        let builder = ExplainerPromptBuilder::ubuntu();
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("ShellCommandExplainer"));
        assert!(prompt.contains("TOOL KNOWLEDGE"));
        assert!(prompt.contains("find"));
        assert!(prompt.contains("grep"));
    }

    #[test]
    fn test_format_chat() {
        let builder = ExplainerPromptBuilder::ubuntu();
        let chat = builder.format_chat("find files modified today");

        assert!(chat.starts_with("<|im_start|>system"));
        assert!(chat.contains("find files modified today"));
        assert!(chat.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn test_create_explanation() {
        let builder = ExplainerPromptBuilder::ubuntu();
        let explanation =
            builder.create_explanation("find . -type f -mtime -1", "find recent files");

        assert_eq!(explanation.tool_used, "find");
        assert!(!explanation.option_breakdown.is_empty());
        assert!(explanation.option_breakdown.iter().any(|o| o.option == "-type f"));
    }

    #[test]
    fn test_identify_tool() {
        let builder = ExplainerPromptBuilder::ubuntu();

        assert_eq!(
            builder.identify_primary_tool("find . -type f"),
            "find"
        );
        assert_eq!(
            builder.identify_primary_tool("grep -rn 'pattern' ."),
            "grep"
        );
        assert_eq!(
            builder.identify_primary_tool("ls -la"),
            "ls"
        );
    }
}
