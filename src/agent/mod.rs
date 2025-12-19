use crate::backends::{CommandGenerator, GeneratorError};
use crate::context::ExecutionContext;
use crate::models::{CommandRequest, GeneratedCommand, SafetyLevel, ShellType};
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
    max_iterations: usize,
    timeout: Duration,
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
            max_iterations: 2,
            timeout: Duration::from_secs(15), // Allow enough time for 2 iterations
        }
    }

    /// Generate command with iterative refinement
    pub async fn generate_command(&self, prompt: &str) -> Result<GeneratedCommand, GeneratorError> {
        let start = Instant::now();

        info!("Starting agent loop for: {}", prompt);

        // Iteration 1: Initial generation with platform context
        debug!("Iteration 1: Generating initial command");
        let initial = self.generate_initial(prompt).await?;

        debug!("Initial command: {}", initial.command);

        // Check if we have time and should refine
        let elapsed = start.elapsed();
        if elapsed > self.timeout / 2 {
            warn!("Timeout approaching, skipping refinement");
            return Ok(initial);
        }

        // Check if refinement is beneficial
        if !self.should_refine(&initial) {
            info!("Refinement not needed, returning initial command");
            return Ok(initial);
        }

        // Iteration 2: Refine with command context
        debug!("Iteration 2: Refining with command context");
        let commands = Self::extract_commands(&initial.command);
        let command_context = self.get_command_context(&commands).await;

        let refined = self
            .refine_command(prompt, &initial, &command_context)
            .await?;

        info!("Command generation complete in {:?}", start.elapsed());
        Ok(refined)
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
    async fn refine_command(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &HashMap<String, CommandInfo>,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let system_prompt = self.build_refinement_prompt(prompt, initial, command_context);

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
    fn build_refinement_prompt(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &HashMap<String, CommandInfo>,
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

        format!(
            r#"COMMAND REFINEMENT ITERATION

ORIGINAL REQUEST: {}

INITIAL COMMAND: {}

COMMAND DETAILS FOR YOUR PLATFORM ({}):
{}

TASK: Review and refine the command if needed.

COMMON ISSUES TO CHECK:
1. Platform-specific flags (--sort vs pipe to sort)
2. Command availability (ss vs lsof vs netstat)
3. Correct syntax for version installed
4. Proper quoting and escaping
5. Path assumptions (/ vs . vs ~/)

OUTPUT FORMAT (JSON):
{{
  "cmd": "refined command (or same if no changes needed)",
  "confidence": 0.98,
  "changes": "brief description of what was fixed"
}}

If the initial command is correct, return it with confidence > 0.9.
If you made changes, explain what was fixed."#,
            prompt, initial.command, self.context.os, command_details
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
        let parts: Vec<&str> = command
            .split(|c| c == '|' || c == ';' || c == '&')
            .collect();

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
        assert_eq!(commands, vec!["find", "xargs", "grep"]);
    }
}
