//! SmolLM-135M-Instruct Optimized Prompt System
//!
//! This module provides a comprehensive system prompt optimized for small language models
//! like SmolLM-135M-Instruct (and similar models in the 100M-500M parameter range).
//!
//! # Design Principles
//!
//! 1. **Explicit and Compact**: Small models need clear, unambiguous instructions
//! 2. **Template-Driven**: Use decision procedures and templates rather than open-ended generation
//! 3. **Platform-Aware**: Generate commands compatible with detected capabilities
//! 4. **Safety-First**: Default to non-destructive operations
//! 5. **Deterministic Output**: Strict schema for consistent parsing
//!
//! # Chat Template Format
//!
//! SmolLM-Instruct uses ChatML format:
//! ```text
//! <|im_start|>system
//! {system_prompt}
//! <|im_end|>
//! <|im_start|>user
//! {user_message}
//! <|im_end|>
//! <|im_start|>assistant
//! ```
//!
//! # Example Usage
//!
//! ```rust
//! use caro::prompts::smollm_prompt::SmolLMPromptBuilder;
//! use caro::prompts::capability_profile::CapabilityProfile;
//!
//! let profile = CapabilityProfile::ubuntu();
//! let builder = SmolLMPromptBuilder::new(profile);
//! let prompt = builder.build_system_prompt();
//! let full_prompt = builder.format_chat("list all files");
//! ```

use super::capability_profile::{CapabilityProfile, ProfileType, StatFormat};
use super::command_templates::TemplateLibrary;

/// Builder for SmolLM-optimized prompts
pub struct SmolLMPromptBuilder {
    profile: CapabilityProfile,
    template_library: TemplateLibrary,
    max_pipeline_stages: usize,
    allow_destructive: bool,
    current_directory: Option<String>,
    available_context: Option<String>,
}

impl SmolLMPromptBuilder {
    /// Create a new prompt builder with the given capability profile
    pub fn new(profile: CapabilityProfile) -> Self {
        let template_library = TemplateLibrary::for_profile(&profile);

        Self {
            profile,
            template_library,
            max_pipeline_stages: 4,
            allow_destructive: false,
            current_directory: None,
            available_context: None,
        }
    }

    /// Create an Ubuntu-optimized prompt builder
    pub fn ubuntu() -> Self {
        Self::new(CapabilityProfile::ubuntu())
    }

    /// Set maximum allowed pipeline stages
    pub fn max_pipeline_stages(mut self, stages: usize) -> Self {
        self.max_pipeline_stages = stages;
        self
    }

    /// Allow destructive commands (rm, mv to overwrite, etc.)
    pub fn allow_destructive(mut self, allow: bool) -> Self {
        self.allow_destructive = allow;
        self
    }

    /// Set current working directory context
    pub fn current_directory(mut self, dir: impl Into<String>) -> Self {
        self.current_directory = Some(dir.into());
        self
    }

    /// Set additional context (e.g., previous command output)
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.available_context = Some(context.into());
        self
    }

    /// Build the complete system prompt
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = String::new();

        // Role and goal
        prompt.push_str(&self.build_role_section());
        prompt.push('\n');

        // Output format rules
        prompt.push_str(&self.build_output_rules());
        prompt.push('\n');

        // Decision procedure
        prompt.push_str(&self.build_decision_procedure());
        prompt.push('\n');

        // Command templates
        prompt.push_str(&self.build_template_section());
        prompt.push('\n');

        // Environment and capabilities
        prompt.push_str(&self.build_environment_section());
        prompt.push('\n');

        // Toolbox
        prompt.push_str(&self.build_toolbox_section());
        prompt.push('\n');

        // Safety rules
        prompt.push_str(&self.build_safety_section());
        prompt.push('\n');

        // Few-shot examples
        prompt.push_str(&self.build_examples_section());

        prompt.push_str("\nRespond to the next user message using the rules above.\n");

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

    /// Format a chat prompt with JSON output requirement
    pub fn format_chat_json(&self, user_query: &str) -> String {
        let system_prompt = self.build_system_prompt();

        format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n{{\"cmd\": \"",
            system_prompt, user_query
        )
    }

    fn build_role_section(&self) -> String {
        r#"You are ShellCommandSynthesizer, an assistant that converts natural language to Unix shell commands.

GOAL: Given a user's intent, output the most relevant shell command(s) for their environment.
OUTPUT: The command itself IS the answer. Focus on WHAT the user wants done."#
            .to_string()
    }

    fn build_output_rules(&self) -> String {
        let destructive_rule = if self.allow_destructive {
            "Destructive commands ARE allowed (user has confirmed)."
        } else {
            "If destructive action implied (rm/mv/chmod/dd), output: QUESTION: <confirm intent>"
        };

        format!(
            r#"OUTPUT FORMAT (strict):
1. Default: Output ONLY JSON: {{"cmd": "your_command_here"}}
2. If clarification needed: Output ONLY: QUESTION: <one short question>
3. Never add explanations, commentary, or alternatives.
4. Never output multiple commands unless piped/chained.
5. {}
6. Max {} pipeline stages allowed."#,
            destructive_rule, self.max_pipeline_stages
        )
    }

    fn build_decision_procedure(&self) -> String {
        let mut procedure = String::from("DECISION PROCEDURE:\n");

        procedure.push_str(
            r#"1. Parse intent -> determine category:
   - "list/show files" -> LISTING
   - "find files with condition" -> FILTERING (use find)
   - "search for text/pattern" -> TEXT_SEARCH (use grep)
   - "largest/newest/top N" -> RANKING (find/ls + sort + head)
   - "count" -> COUNTING (pipe to wc -l)
   - "change/modify/delete" -> MUTATING (confirm if destructive)

2. Select template from TEMPLATES section matching category + profile
3. Fill template with user's parameters
4. Verify all flags are in CAPABILITY_PROFILE
5. Output JSON"#,
        );

        procedure
    }

    fn build_template_section(&self) -> String {
        let mut section = String::from("[TEMPLATES]\n");
        section.push_str(&format!("Profile: {}\n\n", self.profile.profile_type));

        // Group templates by category
        let templates = self.template_library.all_templates();

        let mut current_category = String::new();
        for template in templates {
            if template.category != current_category {
                current_category = template.category.clone();
                section.push_str(&format!("# {}\n", current_category));
            }
            section.push_str(&format!(
                "{}: {}\n",
                template.intent_pattern, template.command_template
            ));
        }

        section
    }

    fn build_environment_section(&self) -> String {
        let cwd = self.current_directory.as_deref().unwrap_or("(current)");

        let mut section = format!(
            r#"[ENVIRONMENT]
OS: {} {}
Profile: {}
Shell: {}
Working Directory: {}
"#,
            self.profile.os_name,
            self.profile.os_version,
            self.profile.profile_type,
            self.profile.detected_shell,
            cwd
        );

        // Add capability profile
        section.push_str("\n[CAPABILITY_PROFILE]\n");
        section.push_str(&format!("FIND_PRINTF={}\n", self.profile.find_printf));
        section.push_str(&format!("SORT_H={}\n", self.profile.sort_h));
        section.push_str(&format!("XARGS_0={}\n", self.profile.xargs_0));
        section.push_str(&format!("GREP_R={}\n", self.profile.grep_r));
        section.push_str(&format!("GREP_P={}\n", self.profile.grep_p));
        section.push_str(&format!("STAT_FORMAT={}\n", self.profile.stat_format));
        section.push_str(&format!(
            "SED_INPLACE_GNU={}\n",
            self.profile.sed_inplace_gnu
        ));
        section.push_str(&format!("DU_MAX_DEPTH={}\n", self.profile.du_max_depth));
        section.push_str(&format!("DATE_GNU={}\n", self.profile.date_gnu_format));
        section.push_str(&format!("PS_SORT={}\n", self.profile.ps_sort));

        // Add capability notes
        let notes = self.profile.capability_notes();
        if !notes.is_empty() {
            section.push_str("\nNOTES:\n");
            for note in notes {
                section.push_str(&format!("- {}\n", note));
            }
        }

        // Add additional context if available
        if let Some(ctx) = &self.available_context {
            section.push_str(&format!("\n[CONTEXT]\n{}\n", ctx));
        }

        section
    }

    fn build_toolbox_section(&self) -> String {
        let mut section = String::from("[TOOLBOX]\n");
        section.push_str("Only use these commands:\n");

        // Core tools always available
        let core_tools = self.get_core_tools_description();
        for (tool, desc) in core_tools {
            if self.profile.tools.contains(&tool.to_string()) || self.profile.tools.is_empty() {
                section.push_str(&format!("- {}: {}\n", tool, desc));
            }
        }

        section
    }

    fn get_core_tools_description(&self) -> Vec<(&'static str, String)> {
        let mut tools = vec![
            ("ls", self.ls_description()),
            ("find", self.find_description()),
            ("grep", self.grep_description()),
            ("awk", "field processing, pattern matching".to_string()),
            ("sed", self.sed_description()),
            ("sort", self.sort_description()),
            ("head", "first N lines: head -n N".to_string()),
            ("tail", "last N lines: tail -n N".to_string()),
            ("wc", "count lines/words/bytes: wc -l".to_string()),
            ("xargs", self.xargs_description()),
            ("cat", "print file contents".to_string()),
            ("cut", "extract fields: cut -d',' -f1".to_string()),
            ("tr", "translate characters: tr 'a-z' 'A-Z'".to_string()),
            ("uniq", "unique lines (input must be sorted)".to_string()),
            ("stat", self.stat_description()),
            ("du", self.du_description()),
            ("date", self.date_description()),
        ];

        // Add platform-specific tools
        if self.profile.profile_type == ProfileType::GnuLinux {
            tools.push(("ss", "socket statistics: ss -tuln".to_string()));
        } else {
            tools.push(("netstat", "network stats: netstat -an".to_string()));
            tools.push(("lsof", "list open files: lsof -iTCP".to_string()));
        }

        tools
    }

    fn ls_description(&self) -> String {
        if self.profile.ls_sort {
            "list files: ls -la (--sort=size/time/extension)".to_string()
        } else {
            "list files: ls -la (pipe to sort for ordering)".to_string()
        }
    }

    fn find_description(&self) -> String {
        let mut desc = "find files: -type f/d, -name, -size, -mtime".to_string();
        if self.profile.find_printf {
            desc.push_str(", -printf");
        }
        if self.profile.find_print0 {
            desc.push_str(", -print0");
        }
        desc
    }

    fn grep_description(&self) -> String {
        let mut desc = "search text: -i (ignore case), -n (line nums)".to_string();
        if self.profile.grep_r {
            desc.push_str(", -R (recursive)");
        }
        if self.profile.grep_p {
            desc.push_str(", -P (Perl regex)");
        } else {
            desc.push_str(", -E (extended regex)");
        }
        desc
    }

    fn sed_description(&self) -> String {
        if self.profile.sed_inplace_gnu {
            "stream edit: sed -i 's/old/new/g'".to_string()
        } else {
            "stream edit: sed -i '' 's/old/new/g' (BSD requires '')".to_string()
        }
    }

    fn sort_description(&self) -> String {
        let mut desc = "sort: -n (numeric), -r (reverse)".to_string();
        if self.profile.sort_h {
            desc.push_str(", -h (human-readable)");
        }
        desc
    }

    fn xargs_description(&self) -> String {
        let mut desc = "build args from stdin".to_string();
        if self.profile.xargs_0 {
            desc.push_str(": xargs -0 (null-delimited)");
        }
        desc
    }

    fn stat_description(&self) -> String {
        match self.profile.stat_format {
            StatFormat::Gnu => "file info: stat -c '%s %n' (GNU format)".to_string(),
            StatFormat::Bsd => "file info: stat -f '%z %N' (BSD format)".to_string(),
            StatFormat::None => "file info (basic)".to_string(),
        }
    }

    fn du_description(&self) -> String {
        if self.profile.du_max_depth {
            "disk usage: du -h --max-depth=N".to_string()
        } else {
            "disk usage: du -h -d N (depth N)".to_string()
        }
    }

    fn date_description(&self) -> String {
        if self.profile.date_gnu_format {
            "date: date --date='7 days ago' (GNU)".to_string()
        } else {
            "date: date -v-7d (BSD)".to_string()
        }
    }

    fn build_safety_section(&self) -> String {
        let mut section = String::from("[SAFETY]\n");

        if self.allow_destructive {
            section.push_str("DESTRUCTIVE_CONFIRMED=true\n");
            section.push_str("- User has confirmed destructive operations are OK\n");
        } else {
            section.push_str("DESTRUCTIVE_CONFIRMED=false\n");
            section.push_str(
                r#"- NEVER output: rm -rf /, dd if=/dev/zero, mkfs, :(){ :|:& };:
- For rm/mv/chmod/chown: ask QUESTION first
- Default to non-destructive: prefer ls over rm, cp over mv
"#,
            );
        }

        section.push_str(
            r#"- Quote paths with spaces: "path with spaces"
- Use -print0 | xargs -0 for filenames with special chars"#,
        );

        section
    }

    fn build_examples_section(&self) -> String {
        let mut section = String::from("[EXAMPLES]\n");

        // Select examples based on profile
        let examples = match self.profile.profile_type {
            ProfileType::GnuLinux => self.gnu_examples(),
            ProfileType::Bsd => self.bsd_examples(),
            ProfileType::Busybox => self.busybox_examples(),
            _ => self.posix_examples(),
        };

        for (intent, command) in examples {
            section.push_str(&format!(
                "User: \"{}\"\nOutput: {{\"cmd\": \"{}\"}}\n\n",
                intent, command
            ));
        }

        section
    }

    fn gnu_examples(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            // PRIORITY: Website-advertised examples (must match exactly)
            ("list all files modified today", "find . -type f -mtime 0"),
            ("find large files over 100MB", "find . -type f -size +100M"),
            ("show disk usage by folder", "du -sh */ | sort -rh | head -10"),
            ("find python files modified last week", "find . -name \"*.py\" -type f -mtime -7"),

            // Additional training examples
            ("list all files", "ls -a"),
            (
                "20 most recently modified files",
                "find . -type f -printf '%T@ %p\\n' | sort -nr | head -n 20 | cut -d' ' -f2-",
            ),
            ("find files containing TODO", "grep -R -n 'TODO' ."),
            (
                "count lines of code in python files",
                "find . -name '*.py' -type f | xargs wc -l",
            ),
        ]
    }

    fn bsd_examples(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            // PRIORITY: Website-advertised examples (must match exactly)
            ("list all files modified today", "find . -type f -mtime 0"),
            ("find large files over 100MB", "find . -type f -size +100M"),
            ("show disk usage by folder", "du -sh */ | sort -rh | head -10"),
            ("find python files modified last week", "find . -name \"*.py\" -type f -mtime -7"),

            // Additional training examples
            ("list all files", "ls -a"),
            ("20 most recently modified files", "find . -type f -exec stat -f '%m %N' {} + | sort -nr | head -n 20 | cut -d' ' -f2-"),
            ("find files containing TODO", "grep -R -n 'TODO' ."),
            ("count lines of code in python files", "find . -name '*.py' -type f | xargs wc -l"),
        ]
    }

    fn busybox_examples(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("list all files", "ls -a"),
            ("list files larger than 100MB", "find . -type f -size +100M"),
            ("find files containing TODO", "grep -r 'TODO' ."),
            ("count lines in file", "wc -l file.txt"),
        ]
    }

    fn posix_examples(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("list all files", "ls -a"),
            ("find files by name", "find . -name '*.txt' -type f"),
            ("search for text", "grep -n 'pattern' file.txt"),
            ("count lines", "wc -l file.txt"),
        ]
    }
}

/// Prompt variant for repair/retry after validation failure
pub struct RepairPromptBuilder {
    original_query: String,
    failed_command: String,
    error_message: String,
    profile: CapabilityProfile,
}

impl RepairPromptBuilder {
    pub fn new(
        original_query: impl Into<String>,
        failed_command: impl Into<String>,
        error_message: impl Into<String>,
        profile: CapabilityProfile,
    ) -> Self {
        Self {
            original_query: original_query.into(),
            failed_command: failed_command.into(),
            error_message: error_message.into(),
            profile,
        }
    }

    pub fn build(&self) -> String {
        format!(
            r#"<|im_start|>system
You are ShellCommandSynthesizer. Fix the command based on the error.
Profile: {}

RULES:
1. Output ONLY: {{"cmd": "fixed_command"}}
2. The previous command failed validation
3. Fix the specific issue mentioned in ERROR
<|im_end|>
<|im_start|>user
Original request: {}
Failed command: {}
ERROR: {}
<|im_end|>
<|im_start|>assistant
"#,
            self.profile.profile_type, self.original_query, self.failed_command, self.error_message
        )
    }
}

/// Output schema for command generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandOutput {
    /// The generated shell command
    pub cmd: String,
}

/// Alternative output for clarification
#[derive(Debug, Clone)]
pub enum PromptResponse {
    /// A successfully generated command
    Command(CommandOutput),
    /// A clarifying question
    Question(String),
    /// Failed to parse response
    ParseError(String),
}

impl PromptResponse {
    /// Parse model output into a structured response
    pub fn parse(output: &str) -> Self {
        let trimmed = output.trim();

        // Check for question format
        if let Some(question) = trimmed.strip_prefix("QUESTION:") {
            return PromptResponse::Question(question.trim().to_string());
        }

        // Try to parse as JSON
        if let Ok(cmd_output) = serde_json::from_str::<CommandOutput>(trimmed) {
            return PromptResponse::Command(cmd_output);
        }

        // Try to extract JSON from response
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                let json_part = &trimmed[start..=end];
                if let Ok(cmd_output) = serde_json::from_str::<CommandOutput>(json_part) {
                    return PromptResponse::Command(cmd_output);
                }
            }
        }

        // Try to extract command from partial JSON like {"cmd": "ls -la
        if let Some(start) = trimmed.find("\"cmd\":") {
            let after_key = &trimmed[start + 6..];
            let after_key = after_key.trim_start();
            if let Some(rest) = after_key.strip_prefix('"') {
                // Find end of string (quote or end of input)
                let cmd = if let Some(end_quote) = rest.find('"') {
                    &rest[..end_quote]
                } else {
                    rest.trim_end_matches('}').trim()
                };
                if !cmd.is_empty() {
                    return PromptResponse::Command(CommandOutput {
                        cmd: cmd.to_string(),
                    });
                }
            }
        }

        PromptResponse::ParseError(format!("Could not parse response: {}", trimmed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ubuntu_prompt_builder() {
        let builder = SmolLMPromptBuilder::ubuntu();
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("ShellCommandSynthesizer"));
        assert!(prompt.contains("FIND_PRINTF=true"));
        assert!(prompt.contains("SORT_H=true"));
        assert!(prompt.contains("[TEMPLATES]"));
        assert!(prompt.contains("[TOOLBOX]"));
    }

    #[test]
    fn test_bsd_prompt_builder() {
        let profile = CapabilityProfile::for_platform(ProfileType::Bsd);
        let builder = SmolLMPromptBuilder::new(profile);
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("FIND_PRINTF=false"));
        assert!(prompt.contains("sed -i ''"));
    }

    #[test]
    fn test_format_chat() {
        let builder = SmolLMPromptBuilder::ubuntu();
        let chat = builder.format_chat("list all files");

        assert!(chat.starts_with("<|im_start|>system"));
        assert!(chat.contains("<|im_start|>user"));
        assert!(chat.contains("list all files"));
        assert!(chat.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn test_parse_command_response() {
        // Valid JSON
        let response = PromptResponse::parse(r#"{"cmd": "ls -la"}"#);
        match response {
            PromptResponse::Command(cmd) => assert_eq!(cmd.cmd, "ls -la"),
            _ => panic!("Expected Command"),
        }

        // Question
        let response = PromptResponse::parse("QUESTION: Do you want to delete all files?");
        match response {
            PromptResponse::Question(q) => assert!(q.contains("delete")),
            _ => panic!("Expected Question"),
        }

        // JSON with extra content
        let response =
            PromptResponse::parse(r#"Here's the command: {"cmd": "find . -name '*.txt'"}"#);
        match response {
            PromptResponse::Command(cmd) => assert_eq!(cmd.cmd, "find . -name '*.txt'"),
            _ => panic!("Expected Command"),
        }

        // Partial JSON (common with small models)
        let response = PromptResponse::parse(r#"{"cmd": "ls -la"#);
        match response {
            PromptResponse::Command(cmd) => assert_eq!(cmd.cmd, "ls -la"),
            _ => panic!("Expected Command from partial JSON"),
        }
    }

    #[test]
    fn test_repair_prompt() {
        let profile = CapabilityProfile::ubuntu();
        let repair = RepairPromptBuilder::new(
            "list files by size",
            "ls --sort=size",
            "flag --sort not available on this system",
            profile,
        );
        let prompt = repair.build();

        assert!(prompt.contains("list files by size"));
        assert!(prompt.contains("ls --sort=size"));
        assert!(prompt.contains("flag --sort not available"));
    }

    #[test]
    fn test_destructive_mode() {
        let builder = SmolLMPromptBuilder::ubuntu().allow_destructive(true);
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("DESTRUCTIVE_CONFIRMED=true"));
        assert!(prompt.contains("Destructive commands ARE allowed"));
    }
}
