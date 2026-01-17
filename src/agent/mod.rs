use crate::backends::{CommandGenerator, GeneratorError, StaticMatcher};
use crate::context::{DirectoryContext, ExecutionContext};
use crate::models::{CommandRequest, GeneratedCommand, SafetyLevel, ShellType};
use crate::prompts::{CapabilityProfile, CommandValidator, ValidationResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

#[cfg(feature = "knowledge")]
use crate::knowledge::{default_knowledge_path, KnowledgeIndex};

/// Agent loop for iterative command refinement
pub struct AgentLoop {
    backend: Arc<dyn CommandGenerator>,
    static_matcher: Option<StaticMatcher>,
    validator: CommandValidator,
    context: ExecutionContext,
    directory_context: DirectoryContext,
    _max_iterations: usize,
    timeout: Duration,
    confidence_threshold: f64,
    /// Knowledge index for learning from past commands (optional)
    #[cfg(feature = "knowledge")]
    knowledge_index: Option<Arc<KnowledgeIndex>>,
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
    pub fn new(
        backend: Arc<dyn CommandGenerator>,
        context: ExecutionContext,
        profile: CapabilityProfile,
    ) -> Self {
        // Create static matcher with detected capabilities
        let static_matcher = Some(StaticMatcher::new(profile.clone()));
        let validator = CommandValidator::new(profile);

        // Scan current directory for project context
        let directory_context = DirectoryContext::scan(context.cwd.as_path());

        Self {
            backend,
            static_matcher,
            validator,
            context,
            directory_context,
            _max_iterations: 2,
            timeout: Duration::from_secs(15), // Allow enough time for 2 iterations
            confidence_threshold: 0.8,        // Default: refine if confidence < 80%
            #[cfg(feature = "knowledge")]
            knowledge_index: None,
        }
    }

    /// Set the confidence threshold for triggering refinement
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    pub fn with_static_matcher(mut self, enabled: bool) -> Self {
        if !enabled {
            self.static_matcher = None;
        }
        self
    }

    /// Enable the knowledge index for learning from past commands
    ///
    /// This initializes a local vector database that stores successful
    /// commands and corrections, improving future suggestions.
    #[cfg(feature = "knowledge")]
    pub async fn with_knowledge(mut self) -> Self {
        match KnowledgeIndex::open(&default_knowledge_path()).await {
            Ok(index) => {
                info!("Knowledge index initialized");
                self.knowledge_index = Some(Arc::new(index));
            }
            Err(e) => {
                warn!("Failed to initialize knowledge index: {}", e);
            }
        }
        self
    }

    /// Enable the knowledge index with a custom path
    #[cfg(feature = "knowledge")]
    pub async fn with_knowledge_path(mut self, path: &std::path::Path) -> Self {
        match KnowledgeIndex::open(path).await {
            Ok(index) => {
                info!("Knowledge index initialized at {:?}", path);
                self.knowledge_index = Some(Arc::new(index));
            }
            Err(e) => {
                warn!("Failed to initialize knowledge index at {:?}: {}", path, e);
            }
        }
        self
    }

    /// Generate command with iterative refinement
    pub async fn generate_command(&self, prompt: &str) -> Result<GeneratedCommand, GeneratorError> {
        let start = Instant::now();
        let result = self.generate_command_impl(prompt, start).await;

        // Emit telemetry on error
        if let Err(ref e) = result {
            let error_category = match e {
                GeneratorError::Timeout { .. } => "timeout",
                GeneratorError::ParseError { .. } => "parse_error",
                GeneratorError::GenerationFailed { .. } => "generation_failed",
                GeneratorError::BackendUnavailable { .. } => "backend_unavailable",
                GeneratorError::InvalidRequest { .. } => "invalid_request",
                GeneratorError::ConfigError { .. } => "config_error",
                GeneratorError::Internal { .. } => "internal_error",
                GeneratorError::Unsafe { .. } => "unsafe_command",
                GeneratorError::ValidationFailed { .. } => "validation_failed",
                GeneratorError::NoMatch { .. } => "no_match",
            };

            crate::telemetry::emit_event(crate::telemetry::events::EventType::CommandGeneration {
                backend: "embedded".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
                success: false,
                error_category: Some(error_category.to_string()),
            });
        }

        // Record successful command to knowledge index
        #[cfg(feature = "knowledge")]
        if let Ok(ref cmd) = result {
            self.record_success(prompt, &cmd.command).await;
        }

        result
    }

    /// Record a successful command to the knowledge index
    #[cfg(feature = "knowledge")]
    async fn record_success(&self, prompt: &str, command: &str) {
        let Some(ref index) = self.knowledge_index else {
            return;
        };

        // Get context (current directory as project context)
        let context = self.context.cwd.to_string_lossy().to_string();

        if let Err(e) = index.record_success(prompt, command, Some(&context)).await {
            debug!("Failed to record command to knowledge index: {}", e);
        } else {
            debug!("Recorded successful command to knowledge index");
        }
    }

    /// Record a correction (original command improved) to the knowledge index
    #[cfg(feature = "knowledge")]
    async fn record_correction(
        &self,
        prompt: &str,
        original: &str,
        corrected: &str,
        feedback: Option<&str>,
    ) {
        let Some(ref index) = self.knowledge_index else {
            return;
        };

        if let Err(e) = index
            .record_correction(prompt, original, corrected, feedback)
            .await
        {
            debug!("Failed to record correction to knowledge index: {}", e);
        } else {
            debug!(
                "Recorded correction to knowledge index: {} -> {}",
                original, corrected
            );
        }
    }

    /// Internal implementation of command generation
    async fn generate_command_impl(
        &self,
        prompt: &str,
        start: Instant,
    ) -> Result<GeneratedCommand, GeneratorError> {
        info!("Starting agent loop for: {}", prompt);

        // Try static matcher first (instant, deterministic)
        if let Some(ref matcher) = self.static_matcher {
            debug!("Trying static pattern matcher");
            let request = CommandRequest::new(prompt, ShellType::Bash);

            match matcher.generate_command(&request).await {
                Ok(command) => {
                    info!(
                        "Static matcher found match in {:?}: {}",
                        start.elapsed(),
                        command.command
                    );

                    // Emit telemetry event for successful static match
                    crate::telemetry::emit_event(
                        crate::telemetry::events::EventType::CommandGeneration {
                            backend: "static".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                            success: true,
                            error_category: None,
                        },
                    );

                    return Ok(command);
                }
                Err(e) => {
                    debug!("Static matcher: no match ({}), falling back to LLM", e);
                }
            }
        }

        // Iteration 1: Initial generation with platform context (LLM fallback)
        debug!("Iteration 1: Generating initial command");
        let initial = self.generate_initial(prompt).await?;

        debug!("Initial command: {}", initial.command);

        // Validate the generated command
        let validation = self.validator.validate(&initial.command);

        // If validation fails, attempt to repair
        if !validation.is_valid() {
            warn!(
                "Initial command failed validation: {}",
                validation.error_message()
            );

            // Check if we have time for repair
            let elapsed = start.elapsed();
            if elapsed > self.timeout / 2 {
                warn!("Timeout approaching, skipping repair");
                // Return error with validation details
                return Err(GeneratorError::ValidationFailed {
                    reason: validation.error_message(),
                });
            }

            // Attempt to repair the command
            debug!("Attempting to repair command with validation feedback");
            let repaired = self.repair_command(prompt, &initial, &validation).await?;

            // Validate the repaired command
            let repaired_validation = self.validator.validate(&repaired.command);
            if !repaired_validation.is_valid() {
                warn!(
                    "Repaired command still invalid: {}",
                    repaired_validation.error_message()
                );
                return Err(GeneratorError::ValidationFailed {
                    reason: format!(
                        "Initial: {}; After repair: {}",
                        validation.error_message(),
                        repaired_validation.error_message()
                    ),
                });
            }

            info!("Command repaired successfully");

            // Record correction to knowledge index
            #[cfg(feature = "knowledge")]
            self.record_correction(
                prompt,
                &initial.command,
                &repaired.command,
                Some(&validation.error_message()),
            )
            .await;

            return Ok(repaired);
        }

        // Check if we have time and should refine
        let elapsed = start.elapsed();
        if elapsed > self.timeout / 2 {
            warn!("Timeout approaching, skipping refinement");
            return Ok(initial);
        }

        // Check confidence score - trigger refinement if low
        let low_confidence = initial.confidence_score < self.confidence_threshold;

        if low_confidence {
            info!(
                "Low confidence ({:.2}), triggering refinement",
                initial.confidence_score
            );
        }

        // Check if refinement is beneficial
        let needs_platform_fix = self.should_refine(&initial);

        if !low_confidence && !needs_platform_fix {
            info!(
                "Refinement not needed (confidence: {:.2}, no platform issues)",
                initial.confidence_score
            );
            return Ok(initial);
        }

        // Iteration 2: Refine with command context
        if low_confidence {
            debug!(
                "Iteration 2: Refining due to low confidence ({:.2})",
                initial.confidence_score
            );
        } else {
            debug!("Iteration 2: Refining due to platform issues");
        }
        let commands = Self::extract_commands(&initial.command);
        let command_context = self.get_command_context(&commands).await;

        let refined = self
            .refine_command(prompt, &initial, &command_context)
            .await?;

        info!("Command generation complete in {:?}", start.elapsed());

        // Record correction if the command was changed during refinement
        #[cfg(feature = "knowledge")]
        if refined.command != initial.command {
            let feedback = if low_confidence {
                "Refined due to low confidence"
            } else {
                "Refined due to platform compatibility"
            };
            self.record_correction(prompt, &initial.command, &refined.command, Some(feedback))
                .await;
        }

        // Emit telemetry event for successful LLM generation
        crate::telemetry::emit_event(crate::telemetry::events::EventType::CommandGeneration {
            backend: "embedded".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
            error_category: None,
        });

        Ok(refined)
    }

    /// Generate initial command with platform context
    async fn generate_initial(&self, prompt: &str) -> Result<GeneratedCommand, GeneratorError> {
        let system_prompt = self.build_initial_prompt();

        // Serialize context to string
        let context_str = serde_json::to_string(&self.context).unwrap_or_else(|_| "{}".to_string());

        // Add directory context if available
        let dir_context_str = if self.directory_context.has_context() {
            format!("\n\n{}", self.directory_context.to_context_string())
        } else {
            String::new()
        };

        // Query knowledge index for similar past commands
        #[cfg(feature = "knowledge")]
        let knowledge_context_str = self.get_knowledge_context(prompt).await;
        #[cfg(not(feature = "knowledge"))]
        let knowledge_context_str = String::new();

        let request = CommandRequest {
            input: prompt.to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(format!(
                "{}{}{}\n\nSYSTEM_PROMPT:\n{}",
                context_str, dir_context_str, knowledge_context_str, system_prompt
            )),
            backend_preference: None,
        };

        self.backend.generate_command(&request).await
    }

    /// Query knowledge index for similar past commands
    #[cfg(feature = "knowledge")]
    async fn get_knowledge_context(&self, prompt: &str) -> String {
        let Some(ref index) = self.knowledge_index else {
            return String::new();
        };

        match index.find_similar(prompt, 3).await {
            Ok(entries) if !entries.is_empty() => {
                debug!(
                    "Found {} similar commands in knowledge index",
                    entries.len()
                );
                let mut context = String::from("\n\nPAST SIMILAR COMMANDS:");
                for entry in entries {
                    context.push_str(&format!(
                        "\n- Request: \"{}\"\n  Command: {}\n  Similarity: {:.2}",
                        entry.request, entry.command, entry.similarity
                    ));
                    if let Some(feedback) = entry.feedback {
                        context.push_str(&format!("\n  Note: {}", feedback));
                    }
                }
                context
            }
            Ok(_) => {
                debug!("No similar commands found in knowledge index");
                String::new()
            }
            Err(e) => {
                warn!("Failed to query knowledge index: {}", e);
                String::new()
            }
        }
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

        // Add directory context if available
        let dir_context_str = if self.directory_context.has_context() {
            format!("\n\n{}", self.directory_context.to_context_string())
        } else {
            String::new()
        };

        let request = CommandRequest {
            input: format!("REFINE: {}", prompt),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(format!(
                "{}{}\n\nSYSTEM_PROMPT:\n{}",
                context_str, dir_context_str, system_prompt
            )),
            backend_preference: None,
        };

        self.backend.generate_command(&request).await
    }

    /// Repair command based on validation errors
    async fn repair_command(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        validation: &ValidationResult,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let system_prompt = self.build_repair_prompt(prompt, initial, validation);

        // Serialize context to string
        let context_str = serde_json::to_string(&self.context).unwrap_or_else(|_| "{}".to_string());

        let request = CommandRequest {
            input: format!("REPAIR: {}", prompt),
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

    /// Build repair prompt with validation error feedback
    fn build_repair_prompt(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        validation: &ValidationResult,
    ) -> String {
        let error_details = validation
            .errors
            .iter()
            .enumerate()
            .map(|(i, err)| {
                format!(
                    "{}. [{:?}] {}\n   Context: {}",
                    i + 1,
                    err.code,
                    err.message,
                    err.context.as_deref().unwrap_or("N/A")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let warning_details = if !validation.warnings.is_empty() {
            format!(
                "\n\nWARNINGS (non-fatal):\n{}",
                validation
                    .warnings
                    .iter()
                    .enumerate()
                    .map(|(i, warn)| {
                        format!(
                            "{}. {}\n   Suggestion: {}",
                            i + 1,
                            warn.message,
                            warn.suggestion.as_deref().unwrap_or("N/A")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };

        format!(
            r#"COMMAND REPAIR ITERATION

ORIGINAL REQUEST: {}

GENERATED COMMAND: {}

VALIDATION ERRORS:
{}{}

PLATFORM: {}
RISK LEVEL: {:?}

YOUR TASK: Fix the command to resolve ALL validation errors.

REPAIR GUIDELINES:

1. **Tool Allowlist Errors**: Only use allowed tools (ls, find, grep, awk, sed, sort, head, tail, xargs, cat, wc, cut, tr, uniq, diff, stat, du, df, ps, lsof, netstat, ss, etc.)

2. **Flag Compatibility Errors**:
   - BSD (macOS): NO GNU flags like --sort, --max-depth, -printf
   - Use: ps aux | sort, find . -exec stat, du -d
   - GNU (Linux): Can use GNU flags
   - Check CAPABILITY_PROFILE for supported flags

3. **Dangerous Command Errors**:
   - NEVER generate: rm -rf /, dd of=/dev/, fork bombs, curl | sh
   - For destructive operations, use safer alternatives or require confirmation

4. **Platform-Specific Fixes**:
{}

5. **Syntax Errors**:
   - Properly quote paths with spaces
   - Balance quotes
   - Use single command or pipeline (no multiple commands)

OUTPUT FORMAT (JSON):
{{
  "cmd": "fixed command that resolves all errors"
}}

CRITICAL: Your repaired command MUST:
- Resolve ALL listed validation errors
- Be platform-compatible ({})
- Use only allowed tools
- Follow proper shell syntax

DO NOT include explanations, comments, or multiple commands - just ONE valid command in JSON format."#,
            prompt,
            initial.command,
            error_details,
            warning_details,
            self.context.os,
            validation.risk_level,
            self.get_os_specific_notes(),
            self.context.os
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
