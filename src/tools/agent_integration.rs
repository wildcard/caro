//! Agent Integration - Tool-enhanced command generation
//!
//! Provides integration between the tool system and the agent loop,
//! enabling rich context gathering and safety validation during
//! command generation.

use super::{ToolCall, ToolData, ToolRegistry, ToolResult};
use crate::backends::{CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand, SafetyLevel, ShellType};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Tool-enhanced agent for command generation with rich context
pub struct ToolEnhancedAgent {
    backend: Arc<dyn CommandGenerator>,
    registry: ToolRegistry,
    timeout: Duration,
    safety_threshold: i64,
}

/// Result of tool-enhanced command generation
#[derive(Debug)]
pub struct EnhancedCommandResult {
    pub command: GeneratedCommand,
    pub risk_score: i64,
    pub risk_level: String,
    pub is_safe: bool,
    pub requires_confirmation: bool,
    pub alternatives: Vec<String>,
    pub context_gathered: ContextSummary,
}

/// Summary of context gathered during generation
#[derive(Debug, Default)]
pub struct ContextSummary {
    pub platform: Option<String>,
    pub shell: Option<String>,
    pub cwd: Option<String>,
    pub commands_checked: Vec<String>,
    pub files_validated: Vec<String>,
}

impl ToolEnhancedAgent {
    /// Create a new tool-enhanced agent
    pub fn new(backend: Arc<dyn CommandGenerator>) -> Self {
        Self {
            backend,
            registry: ToolRegistry::default(),
            timeout: Duration::from_secs(15),
            safety_threshold: 50, // Block commands with risk > 50
        }
    }

    /// Set custom tool registry
    pub fn with_registry(mut self, registry: ToolRegistry) -> Self {
        self.registry = registry;
        self
    }

    /// Set timeout for command generation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set safety threshold (0-100)
    pub fn with_safety_threshold(mut self, threshold: i64) -> Self {
        self.safety_threshold = threshold;
        self
    }

    /// Generate command with full tool-enhanced context
    pub async fn generate(&self, prompt: &str) -> Result<EnhancedCommandResult, GeneratorError> {
        let start = Instant::now();
        info!("Starting tool-enhanced generation for: {}", prompt);

        // Phase 1: Gather context using tools
        let context = self.gather_context().await;
        debug!("Context gathered: {:?}", context);

        // Phase 2: Generate command with context
        let command = self.generate_with_context(prompt, &context).await?;
        debug!("Generated command: {}", command.command);

        // Check timeout
        if start.elapsed() > self.timeout / 2 {
            warn!("Timeout approaching, skipping validation");
            return Ok(EnhancedCommandResult {
                command,
                risk_score: 0,
                risk_level: "UNKNOWN".to_string(),
                is_safe: false,
                requires_confirmation: true,
                alternatives: Vec::new(),
                context_gathered: context,
            });
        }

        // Phase 3: Validate generated command using tools
        let validation = self.validate_command(&command.command).await;
        debug!("Validation result: {:?}", validation);

        info!("Tool-enhanced generation complete in {:?}", start.elapsed());

        Ok(EnhancedCommandResult {
            command,
            risk_score: validation.risk_score,
            risk_level: validation.risk_level,
            is_safe: validation.is_safe,
            requires_confirmation: validation.requires_confirmation,
            alternatives: validation.alternatives,
            context_gathered: context,
        })
    }

    /// Gather system context using tools
    async fn gather_context(&self) -> ContextSummary {
        let mut summary = ContextSummary::default();

        // Get full context
        if let Ok(result) = self.registry.get_context().await {
            if let ToolData::Structured(data) = result.data {
                // Extract OS info
                if let Some(os) = data.fields.get("os") {
                    if let Some(os_map) = os.as_object() {
                        summary.platform =
                            os_map.get("os").and_then(|v| v.as_str()).map(String::from);
                    }
                }

                // Extract shell info
                if let Some(shell) = data.fields.get("shell") {
                    if let Some(shell_map) = shell.as_object() {
                        summary.shell = shell_map
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                    }
                }

                // Extract CWD
                if let Some(cwd) = data.fields.get("cwd") {
                    if let Some(cwd_map) = cwd.as_object() {
                        summary.cwd = cwd_map
                            .get("path")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                    }
                }
            }
        }

        summary
    }

    /// Generate command with gathered context
    async fn generate_with_context(
        &self,
        prompt: &str,
        context: &ContextSummary,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Build context string from gathered data
        let context_str = format!(
            "Platform: {}\nShell: {}\nCWD: {}",
            context.platform.as_deref().unwrap_or("unknown"),
            context.shell.as_deref().unwrap_or("unknown"),
            context.cwd.as_deref().unwrap_or("unknown")
        );

        let request = CommandRequest {
            input: prompt.to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(context_str),
            backend_preference: None,
        };

        self.backend.generate_command(&request).await
    }

    /// Validate command using validation tool
    async fn validate_command(&self, command: &str) -> ValidationResult {
        let result = self.registry.validate_command(command).await;

        match result {
            Ok(tool_result) if tool_result.success => self.parse_validation_result(&tool_result),
            _ => ValidationResult {
                risk_score: 0,
                risk_level: "UNKNOWN".to_string(),
                is_safe: false,
                requires_confirmation: true,
                alternatives: Vec::new(),
            },
        }
    }

    /// Parse validation tool result
    fn parse_validation_result(&self, result: &ToolResult) -> ValidationResult {
        if let ToolData::Structured(data) = &result.data {
            let risk_score = data
                .fields
                .get("risk_score")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let risk_level = data
                .fields
                .get("risk_level")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string();

            let is_safe = data
                .fields
                .get("is_safe")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let requires_confirmation = data
                .fields
                .get("requires_confirmation")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let alternatives = data
                .fields
                .get("safe_alternatives")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            ValidationResult {
                risk_score,
                risk_level,
                is_safe,
                requires_confirmation,
                alternatives,
            }
        } else {
            ValidationResult::default()
        }
    }

    /// Check if specific commands are available
    pub async fn check_commands(&self, commands: Vec<String>) -> Vec<(String, bool)> {
        let mut results = Vec::new();

        for cmd in commands {
            let available = self.registry.command_available(&cmd).await.unwrap_or(false);
            results.push((cmd, available));
        }

        results
    }

    /// Check if paths exist and are safe
    pub async fn check_paths(&self, paths: Vec<String>) -> Vec<PathCheckResult> {
        let mut results = Vec::new();

        for path in paths {
            let call = ToolCall::new("filesystem")
                .with_param("operation", "is_safe")
                .with_path("path", &path);

            if let Ok(result) = self.registry.invoke(&call).await {
                if let ToolData::Structured(data) = result.data {
                    let is_safe = data
                        .fields
                        .get("is_safe")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    let is_system = data
                        .fields
                        .get("is_system_path")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    results.push(PathCheckResult {
                        path: path.clone(),
                        is_safe,
                        is_system_path: is_system,
                    });
                }
            }
        }

        results
    }

    /// Get platform-specific flags for a command
    pub async fn get_platform_flags(&self, command: &str) -> Option<PlatformFlags> {
        let call = ToolCall::new("command")
            .with_param("operation", "platform_flags")
            .with_param("command", command);

        if let Ok(result) = self.registry.invoke(&call).await {
            if let ToolData::Structured(data) = result.data {
                return Some(PlatformFlags {
                    command: command.to_string(),
                    command_type: data
                        .fields
                        .get("type")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    note: data
                        .fields
                        .get("note")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    flags: data.fields.clone(),
                });
            }
        }

        None
    }
}

#[derive(Debug, Default)]
struct ValidationResult {
    risk_score: i64,
    risk_level: String,
    is_safe: bool,
    requires_confirmation: bool,
    alternatives: Vec<String>,
}

#[derive(Debug)]
pub struct PathCheckResult {
    pub path: String,
    pub is_safe: bool,
    pub is_system_path: bool,
}

#[derive(Debug)]
pub struct PlatformFlags {
    pub command: String,
    pub command_type: Option<String>,
    pub note: Option<String>,
    pub flags: std::collections::HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most tests require a backend, so we test utility functions here

    #[test]
    fn test_context_summary_default() {
        let summary = ContextSummary::default();
        assert!(summary.platform.is_none());
        assert!(summary.shell.is_none());
        assert!(summary.commands_checked.is_empty());
    }

    #[tokio::test]
    async fn test_registry_integration() {
        let registry = ToolRegistry::default();

        // Test that registry is properly initialized
        assert!(registry.get("filesystem").is_some());
        assert!(registry.get("command").is_some());
        assert!(registry.get("validation").is_some());
        assert!(registry.get("context").is_some());
    }
}
