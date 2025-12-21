use crate::backends::{CommandGenerator, GeneratorError};
use crate::context::ExecutionContext;
use crate::models::{CommandRequest, GeneratedCommand, ShellType, SafetyLevel};
use crate::preferences::{CommandTranslator, PreferenceCompliance, RawPreferenceData, UserPreferences};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Agent loop for iterative command refinement
pub struct AgentLoop {
    backend: Arc<dyn CommandGenerator>,
    context: ExecutionContext,
    max_iterations: usize,
    timeout: Duration,
    /// User preferences for anti-hallucination
    preferences: Option<UserPreferences>,
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
            max_iterations: 3, // Allow for preference refinement iteration
            timeout: Duration::from_secs(20),
            preferences: None,
        }
    }

    /// Set user preferences for anti-hallucination validation
    pub fn with_preferences(mut self, preferences: UserPreferences) -> Self {
        self.preferences = Some(preferences);
        self
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
        if elapsed > self.timeout / 3 {
            warn!("Timeout approaching, skipping refinement");
            return self.apply_preference_translation(initial);
        }

        // Iteration 2: Check preference compliance (anti-hallucination)
        let (command_after_prefs, pref_compliance) = self.apply_preference_refinement(&initial);

        // Check if refinement is beneficial
        if !self.should_refine(&command_after_prefs) && pref_compliance.map_or(true, |c| c.confidence >= 0.8) {
            info!("Refinement not needed, returning command");
            return Ok(command_after_prefs);
        }

        // Check time again
        let elapsed = start.elapsed();
        if elapsed > self.timeout * 2 / 3 {
            warn!("Timeout approaching, skipping platform refinement");
            return Ok(command_after_prefs);
        }

        // Iteration 3: Refine with command context (platform-specific)
        debug!("Iteration 3: Refining with command context");
        let commands = Self::extract_commands(&command_after_prefs.command);
        let command_context = self.get_command_context(&commands).await;

        let refined = self.refine_command(prompt, &command_after_prefs, &command_context).await?;

        info!("Command generation complete in {:?}", start.elapsed());
        Ok(refined)
    }

    /// Apply preference-based refinement (anti-hallucination)
    ///
    /// This implements the two-layer approach:
    /// 1. Static deterministic rules (CommandTranslator)
    /// 2. Model fallback with raw data (when static rules don't fully apply)
    fn apply_preference_refinement(&self, command: &GeneratedCommand) -> (GeneratedCommand, Option<PreferenceCompliance>) {
        let prefs = match &self.preferences {
            Some(p) => p,
            None => return (command.clone(), None),
        };

        // Layer 1: Check static compliance
        let compliance = prefs.check_compliance(&command.command);
        debug!(
            "Preference compliance: confidence={:.2}, checks={}",
            compliance.confidence,
            compliance.checks.len()
        );

        // If compliance is high, no refinement needed
        if compliance.confidence >= 0.8 {
            return (command.clone(), Some(compliance));
        }

        // Layer 1b: Try static translation first (deterministic)
        let translated = CommandTranslator::translate(&command.command, prefs);
        if translated.was_translated {
            info!(
                "Static translation applied: {} changes",
                translated.changes.len()
            );
            for change in &translated.changes {
                debug!("  - {}", change);
            }

            let mut refined = command.clone();
            refined.command = translated.translated;

            // Re-check compliance after translation
            let new_compliance = prefs.check_compliance(&refined.command);
            return (refined, Some(new_compliance));
        }

        // Layer 2: If static rules didn't help, the command might need
        // model-based refinement. We return the compliance info so the
        // caller can decide whether to invoke model refinement.
        //
        // Note: Model refinement would use prefs.to_raw_data().to_model_context()
        // to build context for the refinement prompt.
        (command.clone(), Some(compliance))
    }

    /// Apply simple preference translation without full compliance check
    fn apply_preference_translation(&self, command: GeneratedCommand) -> Result<GeneratedCommand, GeneratorError> {
        let prefs = match &self.preferences {
            Some(p) => p,
            None => return Ok(command),
        };

        let translated = CommandTranslator::translate(&command.command, prefs);
        if translated.was_translated {
            debug!("Quick translation applied: {}", translated.changes.join(", "));
            let mut refined = command;
            refined.command = translated.translated;
            Ok(refined)
        } else {
            Ok(command)
        }
    }
    
    /// Generate initial command with platform context
    async fn generate_initial(&self, prompt: &str) -> Result<GeneratedCommand, GeneratorError> {
        let system_prompt = self.build_initial_prompt();
        
        // Serialize context to string
        let context_str = serde_json::to_string(&self.context)
            .unwrap_or_else(|_| "{}".to_string());
        
        let request = CommandRequest {
            input: prompt.to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(format!("{}\n\nSYSTEM_PROMPT:\n{}", context_str, system_prompt)),
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
        let context_str = serde_json::to_string(&self.context)
            .unwrap_or_else(|_| "{}".to_string());
        
        let request = CommandRequest {
            input: format!("REFINE: {}", prompt),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(format!("{}\n\nSYSTEM_PROMPT:\n{}", context_str, system_prompt)),
            backend_preference: None,
        };
        
        self.backend.generate_command(&request).await
    }
    
    /// Build initial system prompt with platform context
    fn build_initial_prompt(&self) -> String {
        // Include preference context if available
        let preference_context = self.preferences
            .as_ref()
            .map(|p| p.to_extended_prompt_context())
            .unwrap_or_default();

        format!(r#"You are a shell command generator for {OS}.

**PLATFORM: {OS} - BSD COMMANDS**
{platform_notes}

{preference_section}

CRITICAL RULES:
1. Output ONLY valid JSON: {{"cmd": "command here"}}
2. NEVER use GNU flags (--sort, --max-depth, etc.) on macOS
3. Use BSD-compatible syntax (see platform notes above)
4. Use relative paths (. or ~/) unless absolute path requested
5. Escape quotes properly: use single quotes inside JSON string
{preference_rules}

{context}

RESPONSE FORMAT:
{{
  "cmd": "your command here"
}}

Generate a safe, platform-appropriate command."#,
            OS = self.context.os,
            platform_notes = self.get_os_specific_notes(),
            preference_section = if preference_context.is_empty() {
                String::new()
            } else {
                format!("**USER PREFERENCES**\n{}", preference_context)
            },
            preference_rules = self.get_preference_rules(),
            context = self.context.get_prompt_context()
        )
    }

    /// Get preference-specific rules for the prompt
    fn get_preference_rules(&self) -> String {
        let prefs = match &self.preferences {
            Some(p) => p,
            None => return String::new(),
        };

        let mut rules = Vec::new();

        // Package manager rule
        if let Some(pm) = &prefs.project.package_manager {
            rules.push(format!(
                "6. Use '{}' for package operations (project uses {})",
                pm.command(),
                pm.name()
            ));
        }

        // Alias suggestion
        if !prefs.shell.aliases.is_empty() {
            rules.push("7. Consider user's aliases when generating commands".to_string());
        }

        if rules.is_empty() {
            String::new()
        } else {
            rules.join("\n")
        }
    }
    
    /// Build refinement prompt with command context
    fn build_refinement_prompt(
        &self,
        prompt: &str,
        initial: &GeneratedCommand,
        command_context: &HashMap<String, CommandInfo>,
    ) -> String {
        let command_details = command_context.iter()
            .map(|(name, info)| {
                format!("Command: {}\nVersion: {}\nKey Options:\n{}",
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
        
        format!(r#"COMMAND REFINEMENT ITERATION

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
            prompt,
            initial.command,
            self.context.os,
            command_details
        )
    }
    
    /// Get OS-specific notes for prompt
    fn get_os_specific_notes(&self) -> String {
        match self.context.os.as_str() {
            "macos" => r#"- Use 'ps aux' then pipe to sort (no --sort flag)
- Use 'lsof -iTCP -sTCP:LISTEN' for ports (NOT ss)
- Use 'df -h' then pipe to sort (no --sort flag)
- Use 'find .' not 'find /' to avoid permission errors
- BSD sed: use 'sed -i "" ...' not 'sed -i...'
- Check man pages: commands may have different flags than Linux"#,
            "linux" => r#"- Can use GNU flags: ps --sort, df --sort, etc.
- Use 'ss -tuln' for network ports
- sed: use 'sed -i' for in-place
- Most GNU coreutils available"#,
            _ => "Use POSIX-compliant commands"
        }.to_string()
    }
    
    /// Check if command should be refined
    fn should_refine(&self, command: &GeneratedCommand) -> bool {
        let cmd = &command.command;
        
        // Always refine commands with platform-specific issues
        let has_platform_issues = match self.context.os.as_str() {
            "macos" => {
                cmd.contains("--sort") ||  // GNU-style sorting
                cmd.contains("ss ") ||      // Linux-only command
                cmd.starts_with("find /")   // Permission issues
            },
            _ => false
        };
        
        // Refine if using complex commands
        let uses_complex_commands = cmd.contains("xargs") ||
            cmd.contains("sed") ||
            cmd.contains("awk") ||
            cmd.split('|').count() > 2;
        
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
                if !matches!(cmd, "if" | "then" | "else" | "fi" | "while" | "do" | "done" | ">" | "<" | ">>") {
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
            .map(|s| {
                s.lines()
                    .take(20)
                    .collect::<Vec<_>>()
                    .join("\n")
            });
        
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
    use crate::models::{RiskLevel, ShellType};
    use crate::preferences::{PackageManager, ProjectContext, ShellProfile};
    use chrono::Utc;
    use std::path::PathBuf;

    fn make_test_preferences(pm: PackageManager) -> UserPreferences {
        UserPreferences {
            project: ProjectContext {
                package_manager: Some(pm),
                build_tool: None,
                languages: vec![],
                infra_tools: vec![],
                cloud_context: None,
                root_path: PathBuf::from("/test"),
                detected_files: vec![],
                raw_signals: vec![],
            },
            shell: ShellProfile::empty(ShellType::Bash),
            detected_at: Utc::now(),
            cache_key: "/test".to_string(),
        }
    }

    fn make_test_command(cmd: &str) -> GeneratedCommand {
        GeneratedCommand {
            command: cmd.to_string(),
            explanation: "Test command".to_string(),
            safety_level: RiskLevel::Safe,
            estimated_impact: "None".to_string(),
            alternatives: vec![],
            backend_used: "mock".to_string(),
            generation_time_ms: 0,
            confidence_score: 0.9,
        }
    }

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
        // Note: grep is an argument to xargs, not a separate piped command
        assert_eq!(commands, vec!["find", "xargs"]);
    }

    #[test]
    fn test_preference_rules_with_package_manager() {
        use crate::backends::CommandGenerator;
        use async_trait::async_trait;

        // Mock backend
        struct MockBackend;

        #[async_trait]
        impl CommandGenerator for MockBackend {
            async fn generate_command(
                &self,
                _request: &CommandRequest,
            ) -> Result<GeneratedCommand, GeneratorError> {
                Ok(make_test_command("npm install"))
            }

            async fn is_available(&self) -> bool {
                true
            }

            fn backend_info(&self) -> crate::backends::BackendInfo {
                crate::backends::BackendInfo {
                    backend_type: crate::models::BackendType::Embedded,
                    model_name: "mock".to_string(),
                    supports_streaming: false,
                    max_tokens: 1000,
                    typical_latency_ms: 100,
                    memory_usage_mb: 100,
                    version: "1.0.0".to_string(),
                }
            }

            async fn shutdown(&self) -> Result<(), GeneratorError> {
                Ok(())
            }
        }

        let context = ExecutionContext {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            os_version: "5.15.0".to_string(),
            distribution: Some("Ubuntu 22.04".to_string()),
            cwd: PathBuf::from("/test"),
            shell: "bash".to_string(),
            user: "test".to_string(),
            available_commands: vec!["npm".to_string(), "yarn".to_string()],
        };

        let prefs = make_test_preferences(PackageManager::Yarn);

        let agent = AgentLoop::new(Arc::new(MockBackend), context).with_preferences(prefs);

        let rules = agent.get_preference_rules();
        assert!(rules.contains("yarn"));
        assert!(rules.contains("Yarn"));
    }

    #[test]
    fn test_preference_refinement_translates_npm_to_yarn() {
        use crate::backends::CommandGenerator;
        use async_trait::async_trait;

        struct MockBackend;

        #[async_trait]
        impl CommandGenerator for MockBackend {
            async fn generate_command(
                &self,
                _request: &CommandRequest,
            ) -> Result<GeneratedCommand, GeneratorError> {
                Ok(make_test_command("npm install lodash"))
            }

            async fn is_available(&self) -> bool {
                true
            }

            fn backend_info(&self) -> crate::backends::BackendInfo {
                crate::backends::BackendInfo {
                    backend_type: crate::models::BackendType::Embedded,
                    model_name: "mock".to_string(),
                    supports_streaming: false,
                    max_tokens: 1000,
                    typical_latency_ms: 100,
                    memory_usage_mb: 100,
                    version: "1.0.0".to_string(),
                }
            }

            async fn shutdown(&self) -> Result<(), GeneratorError> {
                Ok(())
            }
        }

        let context = ExecutionContext {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            os_version: "5.15.0".to_string(),
            distribution: Some("Ubuntu 22.04".to_string()),
            cwd: PathBuf::from("/test"),
            shell: "bash".to_string(),
            user: "test".to_string(),
            available_commands: vec!["npm".to_string(), "yarn".to_string()],
        };

        let prefs = make_test_preferences(PackageManager::Yarn);

        let agent = AgentLoop::new(Arc::new(MockBackend), context).with_preferences(prefs);

        // Create a command that uses npm when yarn is preferred
        let command = make_test_command("npm install lodash");

        // Apply preference refinement
        let (refined, compliance) = agent.apply_preference_refinement(&command);

        // Should translate npm to yarn
        assert_eq!(refined.command, "yarn add lodash");
        assert!(compliance.is_some());
    }
}
