use crate::backends::{CommandGenerator, GeneratorError};
use crate::context::ExecutionContext;
use crate::models::{CommandRequest, GeneratedCommand, SafetyLevel, ShellType};
use crate::shellcheck::{AnalysisResult, ShellCheckAnalyzer, ShellDialect};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Agent loop for iterative command refinement
pub struct AgentLoop {
    backend: Arc<dyn CommandGenerator>,
    context: ExecutionContext,
    _max_iterations: usize,
    timeout: Duration,
    /// ShellCheck analyzer for validating generated commands
    shellcheck: ShellCheckAnalyzer,
}

/// Command information for context enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub version: Option<String>,
    pub help_text: Option<String>,
}

/// Response from backend with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub cmd: String,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub commands_used: Vec<String>,
}

impl AgentLoop {
    pub fn new(backend: Arc<dyn CommandGenerator>, context: ExecutionContext) -> Self {
        Self {
            backend,
            context,
            _max_iterations: 2,
            timeout: Duration::from_secs(15), // Allow enough time for 2 iterations
            shellcheck: ShellCheckAnalyzer::new(),
        }
    }

    /// Check if ShellCheck is available for command validation
    pub fn shellcheck_available(&self) -> bool {
        self.shellcheck.is_available()
    }

    /// Generate command with iterative refinement
    pub async fn generate_command(&self, prompt: &str) -> Result<GeneratedCommand, GeneratorError> {
        let start = Instant::now();

        info!("Starting agent loop for: {}", prompt);

        // Iteration 1: Initial generation with platform context
        debug!("Iteration 1: Generating initial command");
        let initial = self.generate_initial(prompt).await?;

        debug!("Initial command: {}", initial.command);

        // Run ShellCheck analysis on the initial command
        let shellcheck_result = self.run_shellcheck_analysis(&initial.command).await;

        // Check if we have time and should refine
        let elapsed = start.elapsed();
        if elapsed > self.timeout / 2 {
            warn!("Timeout approaching, skipping refinement");
            return Ok(initial);
        }

        // Check if refinement is beneficial based on command patterns or ShellCheck issues
        let needs_pattern_refinement = self.should_refine(&initial);
        let needs_shellcheck_refinement = shellcheck_result
            .as_ref()
            .map(|r| r.needs_regeneration())
            .unwrap_or(false);

        if !needs_pattern_refinement && !needs_shellcheck_refinement {
            info!("Refinement not needed, returning initial command");
            return Ok(initial);
        }

        // Log why we're refining
        if needs_shellcheck_refinement {
            if let Some(ref result) = shellcheck_result {
                let (errors, warnings, _, _) = result.count_by_severity();
                info!(
                    "ShellCheck found {} error(s) and {} warning(s), triggering refinement",
                    errors, warnings
                );
            }
        }

        // Iteration 2: Refine with command context and ShellCheck feedback
        debug!("Iteration 2: Refining with command context");
        let commands = Self::extract_commands(&initial.command);
        let command_context = self.get_command_context(&commands).await;

        let refined = self
            .refine_command_with_shellcheck(prompt, &initial, &command_context, &shellcheck_result)
            .await?;

        // Optionally verify the refined command with ShellCheck
        if self.shellcheck.is_available() {
            let refined_check = self.run_shellcheck_analysis(&refined.command).await;
            if let Some(ref result) = refined_check {
                if result.needs_regeneration() {
                    debug!(
                        "Refined command still has ShellCheck issues: {}",
                        result.to_prompt_feedback()
                    );
                } else {
                    debug!("Refined command passed ShellCheck validation");
                }
            }
        }

        info!("Command generation complete in {:?}", start.elapsed());
        Ok(refined)
    }

    /// Run ShellCheck analysis on a command
    async fn run_shellcheck_analysis(&self, command: &str) -> Option<AnalysisResult> {
        if !self.shellcheck.is_available() {
            debug!("ShellCheck not available, skipping analysis");
            return None;
        }

        let dialect = self.get_shell_dialect();

        match self.shellcheck.analyze(command, dialect).await {
            Ok(result) => {
                if !result.issues.is_empty() {
                    debug!(
                        "ShellCheck found {} issue(s) in command",
                        result.issues.len()
                    );
                }
                Some(result)
            }
            Err(e) => {
                warn!("ShellCheck analysis failed: {}", e);
                None
            }
        }
    }

    /// Get the shell dialect for ShellCheck based on context
    fn get_shell_dialect(&self) -> ShellDialect {
        // Default to bash for now, could be enhanced to detect from context
        ShellDialect::Bash
    }

    /// Generate initial command with platform context
    async fn generate_initial(&self, prompt: &str) -> Result<GeneratedCommand, GeneratorError> {
        let system_prompt = self.build_initial_prompt();

        // Serialize context to string
        let context_str = serde_json::to_string(&self.context).unwrap_or_else(|_| "{}".to_string());

        let request = CommandRequest {
            input: prompt.to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(format!(
                "{}\n\nSYSTEM_PROMPT:\n{}",
                context_str, system_prompt
            )),
            backend_preference: None,
        };

        self.backend.generate_command(&request).await
    }

    /// Refine command with command-specific context
    #[allow(dead_code)]
    async fn refine_command(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &HashMap<String, CommandInfo>,
    ) -> Result<GeneratedCommand, GeneratorError> {
        self.refine_command_with_shellcheck(prompt, initial, command_context, &None)
            .await
    }

    /// Refine command with command-specific context and ShellCheck feedback
    async fn refine_command_with_shellcheck(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &HashMap<String, CommandInfo>,
        shellcheck_result: &Option<AnalysisResult>,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let system_prompt =
            self.build_refinement_prompt_with_shellcheck(prompt, initial, command_context, shellcheck_result);

        // Serialize context to string
        let context_str = serde_json::to_string(&self.context).unwrap_or_else(|_| "{}".to_string());

        let request = CommandRequest {
            input: format!("REFINE: {}", prompt),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(format!(
                "{}\n\nSYSTEM_PROMPT:\n{}",
                context_str, system_prompt
            )),
            backend_preference: None,
        };

        self.backend.generate_command(&request).await
    }

    /// Build initial system prompt with platform context
    fn build_initial_prompt(&self) -> String {
        format!(
            r#"You are a shell command generator for {OS}.

**PLATFORM: {OS} - BSD COMMANDS**
{platform_notes}

CRITICAL RULES:
1. Output ONLY valid JSON: {{"cmd": "command here"}}
2. NEVER use GNU flags (--sort, --max-depth, etc.) on macOS
3. Use BSD-compatible syntax (see platform notes above)
4. Use relative paths (. or ~/) unless absolute path requested
5. Escape quotes properly: use single quotes inside JSON string

{context}

RESPONSE FORMAT:
{{
  "cmd": "your command here"
}}

Generate a safe, platform-appropriate command."#,
            OS = self.context.os,
            platform_notes = self.get_os_specific_notes(),
            context = self.context.get_prompt_context()
        )
    }

    /// Build refinement prompt with command context
    #[allow(dead_code)]
    fn build_refinement_prompt(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &HashMap<String, CommandInfo>,
    ) -> String {
        self.build_refinement_prompt_with_shellcheck(prompt, initial, command_context, &None)
    }

    /// Build refinement prompt with command context and ShellCheck feedback
    fn build_refinement_prompt_with_shellcheck(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &HashMap<String, CommandInfo>,
        shellcheck_result: &Option<AnalysisResult>,
    ) -> String {
        let command_details = command_context
            .iter()
            .map(|(name, info)| {
                format!(
                    "Command: {}\nVersion: {}\nKey Options:\n{}",
                    name,
                    info.version.as_deref().unwrap_or("unknown"),
                    info.help_text.as_deref().unwrap_or("No help available")
                        .lines()
                        .take(15)  // First 15 lines of help
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        // Build ShellCheck feedback section if available
        let shellcheck_section = match shellcheck_result {
            Some(result) if result.needs_regeneration() => {
                format!(
                    r#"
SHELLCHECK ANALYSIS (CRITICAL - MUST FIX):
{}
You MUST address these ShellCheck issues in your refined command.
ShellCheck errors and warnings indicate real problems that will cause issues.
"#,
                    result.to_prompt_feedback()
                )
            }
            Some(result) if !result.issues.is_empty() => {
                format!(
                    r#"
SHELLCHECK SUGGESTIONS (optional improvements):
{}
"#,
                    result.to_prompt_feedback()
                )
            }
            _ => String::new(),
        };

        format!(
            r#"COMMAND REFINEMENT ITERATION

ORIGINAL REQUEST: {}

INITIAL COMMAND: {}
{}
COMMAND DETAILS FOR YOUR PLATFORM ({}):
{}

TASK: Review and refine the command if needed.

COMMON ISSUES TO CHECK:
1. Platform-specific flags (--sort vs pipe to sort)
2. Command availability (ss vs lsof vs netstat)
3. Correct syntax for version installed
4. Proper quoting and escaping (ShellCheck SC2086: quote variables)
5. Path assumptions (/ vs . vs ~/)
6. Variable expansion safety (use "$var" not $var)

OUTPUT FORMAT (JSON):
{{
  "cmd": "refined command (or same if no changes needed)",
  "confidence": 0.98,
  "changes": "brief description of what was fixed"
}}

If the initial command is correct, return it with confidence > 0.9.
If you made changes, explain what was fixed."#,
            prompt, initial.command, shellcheck_section, self.context.os, command_details
        )
    }

    /// Get OS-specific notes for prompt
    fn get_os_specific_notes(&self) -> String {
        match self.context.os.as_str() {
            "macos" => {
                r#"- Use 'ps aux' then pipe to sort (no --sort flag)
- Use 'lsof -iTCP -sTCP:LISTEN' for ports (NOT ss)
- Use 'df -h' then pipe to sort (no --sort flag)
- Use 'find .' not 'find /' to avoid permission errors
- BSD sed: use 'sed -i "" ...' not 'sed -i...'
- Check man pages: commands may have different flags than Linux"#
            }
            "linux" => {
                r#"- Can use GNU flags: ps --sort, df --sort, etc.
- Use 'ss -tuln' for network ports
- sed: use 'sed -i' for in-place
- Most GNU coreutils available"#
            }
            _ => "Use POSIX-compliant commands",
        }
        .to_string()
    }

    /// Check if command should be refined
    fn should_refine(&self, command: &GeneratedCommand) -> bool {
        let cmd = &command.command;

        // Always refine commands with platform-specific issues
        let has_platform_issues = match self.context.os.as_str() {
            "macos" => {
                cmd.contains("--sort") ||  // GNU-style sorting
                cmd.contains("ss ") ||      // Linux-only command
                cmd.starts_with("find /") // Permission issues
            }
            _ => false,
        };

        // Refine if using complex commands
        let uses_complex_commands = cmd.contains("xargs")
            || cmd.contains("sed")
            || cmd.contains("awk")
            || cmd.split('|').count() > 2;

        has_platform_issues || uses_complex_commands
    }

    /// Extract command names from shell command
    fn extract_commands(command: &str) -> Vec<String> {
        let mut commands = Vec::new();

        // Split by pipes, semicolons, and logical operators
        let parts: Vec<&str> = command.split(['|', ';', '&']).collect();

        for part in parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Get first word (the command)
            if let Some(cmd) = trimmed.split_whitespace().next() {
                // Skip shell builtins and redirects
                if !matches!(
                    cmd,
                    "if" | "then" | "else" | "fi" | "while" | "do" | "done" | ">" | "<" | ">>"
                ) {
                    commands.push(cmd.to_string());
                }
            }
        }

        commands.into_iter().collect()
    }

    /// Get context information for commands
    async fn get_command_context(&self, commands: &[String]) -> HashMap<String, CommandInfo> {
        let mut context = HashMap::new();

        for cmd in commands {
            if let Ok(info) = Self::get_command_info(cmd).await {
                context.insert(cmd.clone(), info);
            }
        }

        context
    }

    /// Get information about a specific command
    async fn get_command_info(command: &str) -> Result<CommandInfo> {
        // Get version
        let version = Command::new(command)
            .arg("--version")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.lines().next().unwrap_or("").to_string());

        // Get help text (first 20 lines)
        let help_text = Command::new(command)
            .arg("--help")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.lines().take(20).collect::<Vec<_>>().join("\n"));

        Ok(CommandInfo {
            name: command.to_string(),
            version,
            help_text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_commands() {
        let cmd = "ps aux | sort -k3 -rn | head -5";
        let commands = AgentLoop::extract_commands(cmd);
        assert_eq!(commands, vec!["ps", "sort", "head"]);
    }

    #[test]
    fn test_extract_complex_commands() {
        let cmd = "find . -name '*.rs' | xargs grep -l 'TODO'";
        let commands = AgentLoop::extract_commands(cmd);
        // xargs takes grep as an argument, so only find and xargs are top-level commands
        assert_eq!(commands, vec!["find", "xargs"]);
    }
}
