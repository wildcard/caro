//! Inference agent - LLM-powered argument suggestion
//!
//! This module uses the existing CommandGenerator backend to infer appropriate
//! command arguments based on completion context and partial input.

use serde::{Deserialize, Serialize};

use crate::backends::CommandGenerator;
use crate::models::{CommandRequest, ShellType};

use super::context::{ArgumentSpec, CommandContext, CompletionType};
use super::{ArgumentType, AutocompleteError, Candidate};

/// Agent that uses LLM to infer command completions
pub struct InferenceAgent {
    config: InferenceConfig,
    backend: Box<dyn CommandGenerator>,
}

/// Configuration for inference behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Maximum number of candidates to generate
    pub max_candidates: usize,
    /// Temperature for LLM generation (0.0 to 1.0)
    pub temperature: f32,
    /// Whether to include examples in the prompt
    pub include_examples: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_candidates: 10,
            temperature: 0.3,
            include_examples: true,
        }
    }
}

impl InferenceAgent {
    /// Create new inference agent with given backend
    pub fn new(
        config: InferenceConfig,
        backend: Box<dyn CommandGenerator>,
    ) -> Result<Self, AutocompleteError> {
        Ok(Self { config, backend })
    }

    /// Infer completions for partial command
    pub async fn infer_completions(
        &self,
        partial_command: &str,
        cursor_position: usize,
        context: &CommandContext,
    ) -> Result<Vec<Candidate>, AutocompleteError> {
        // Build prompt based on completion type and context
        let prompt = self.build_inference_prompt(partial_command, cursor_position, context);

        // Create request for the backend
        let request = CommandRequest {
            input: prompt,
            shell: ShellType::Bash, // Context-aware in the future
            safety_level: crate::models::SafetyLevel::Moderate,
            context: Some(self.format_context_for_backend(context)),
            backend_preference: None,
        };

        // Get response from LLM backend
        let response = self
            .backend
            .generate_command(&request)
            .await
            .map_err(|e| AutocompleteError::InferenceFailed {
                details: e.to_string(),
            })?;

        // Parse response into candidates
        self.parse_inference_response(&response.command, context)
    }

    /// Build inference prompt based on context
    fn build_inference_prompt(
        &self,
        partial_command: &str,
        cursor_position: usize,
        context: &CommandContext,
    ) -> String {
        let mut prompt = String::new();

        // Add task description
        prompt.push_str("Task: Suggest command completions for the partial command.\n\n");

        // Add partial command
        prompt.push_str(&format!("Partial command: {}\n", partial_command));
        prompt.push_str(&format!("Cursor position: {}\n\n", cursor_position));

        // Add context-specific information
        match context.completion_type {
            CompletionType::Command => {
                prompt.push_str("Context: Completing base command name.\n");
                prompt.push_str("Suggest common shell commands.\n");
            }
            CompletionType::Subcommand => {
                if let Some(sig) = &context.signature {
                    prompt.push_str(&format!("Context: Completing subcommand for '{}'.\n", sig.command));
                    prompt.push_str("Available subcommands:\n");
                    for subcmd in &sig.subcommands {
                        prompt.push_str(&format!("  - {}: {}\n", subcmd.name, subcmd.description));
                    }
                }
            }
            CompletionType::Flag => {
                if let Some(sig) = &context.signature {
                    prompt.push_str(&format!("Context: Completing flag for '{}'.\n", sig.command));

                    // Find current subcommand if any
                    let current_subcmd = context.tokens.get(1).and_then(|token| {
                        sig.subcommands.iter().find(|s| &s.name == token)
                    });

                    if let Some(subcmd) = current_subcmd {
                        prompt.push_str(&format!("Subcommand: {}\n", subcmd.name));
                        prompt.push_str("Available flags:\n");
                        for flag in &subcmd.flags {
                            if let Some(short) = flag.short {
                                prompt.push_str(&format!("  -{}", short));
                            }
                            if let Some(long) = &flag.long {
                                prompt.push_str(&format!("  --{}", long));
                            }
                            prompt.push_str(&format!(": {}\n", flag.description));
                        }
                    }

                    // Add global flags
                    if !sig.global_flags.is_empty() {
                        prompt.push_str("Global flags:\n");
                        for flag in &sig.global_flags {
                            if let Some(long) = &flag.long {
                                prompt.push_str(&format!("  --{}: {}\n", long, flag.description));
                            }
                        }
                    }
                }
            }
            CompletionType::FlagValue => {
                prompt.push_str("Context: Completing value for a flag.\n");

                // Try to find the flag spec
                if let Some(sig) = &context.signature {
                    if context.tokens.len() >= 2 {
                        let flag_token = &context.tokens[context.tokens.len() - 1];

                        // Search for the flag in subcommand or global flags
                        for subcmd in &sig.subcommands {
                            for flag in &subcmd.flags {
                                if flag.matches_token(flag_token) {
                                    if let Some(value_spec) = &flag.value_spec {
                                        prompt.push_str(&format!("Flag: {}\n", flag.description));
                                        prompt.push_str(&format!("Expected value: {}\n", self.format_argument_spec(value_spec)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            CompletionType::Argument => {
                prompt.push_str("Context: Completing positional argument.\n");

                if let Some(sig) = &context.signature {
                    // Find current subcommand
                    let current_subcmd = context.tokens.get(1).and_then(|token| {
                        sig.subcommands.iter().find(|s| &s.name == token)
                    });

                    if let Some(subcmd) = current_subcmd {
                        if !subcmd.arguments.is_empty() {
                            prompt.push_str("Expected arguments:\n");
                            for (i, arg) in subcmd.arguments.iter().enumerate() {
                                prompt.push_str(&format!("  {}: {}\n", i + 1, self.format_argument_spec(arg)));
                            }
                        }
                    }
                }
            }
        }

        // Add instructions for output format
        prompt.push_str("\nOutput format:\n");
        prompt.push_str("Return a JSON array of suggestions. Each suggestion should have:\n");
        prompt.push_str("- \"value\": the completion text\n");
        prompt.push_str("- \"description\": brief explanation\n");
        prompt.push_str("- \"confidence\": score from 0.0 to 1.0\n\n");
        prompt.push_str(&format!("Provide up to {} suggestions.\n", self.config.max_candidates));
        prompt.push_str("JSON array:");

        prompt
    }

    /// Format argument spec for human-readable display
    fn format_argument_spec(&self, spec: &ArgumentSpec) -> String {
        match spec {
            ArgumentSpec::String { pattern, examples } => {
                if let Some(pattern) = pattern {
                    format!("string matching pattern: {}", pattern)
                } else if !examples.is_empty() {
                    format!("string (e.g., {})", examples.join(", "))
                } else {
                    "string value".to_string()
                }
            }
            ArgumentSpec::File { must_exist, extensions } => {
                let mut desc = "file path".to_string();
                if *must_exist {
                    desc.push_str(" (must exist)");
                }
                if let Some(exts) = extensions {
                    desc.push_str(&format!(" [{}]", exts.join(", ")));
                }
                desc
            }
            ArgumentSpec::Directory { must_exist } => {
                let mut desc = "directory path".to_string();
                if *must_exist {
                    desc.push_str(" (must exist)");
                }
                desc
            }
            ArgumentSpec::Enum { values } => {
                format!("one of: {}", values.join(", "))
            }
            ArgumentSpec::Integer { min, max } => {
                let mut desc = "integer".to_string();
                if let Some(min) = min {
                    desc.push_str(&format!(" >= {}", min));
                }
                if let Some(max) = max {
                    desc.push_str(&format!(" <= {}", max));
                }
                desc
            }
            ArgumentSpec::Boolean => "boolean (true/false)".to_string(),
        }
    }

    /// Format context for backend consumption
    fn format_context_for_backend(&self, context: &CommandContext) -> String {
        format!(
            "Command: {}, Completion type: {:?}, Tokens: {}",
            context.base_command,
            context.completion_type,
            context.tokens.join(" ")
        )
    }

    /// Parse LLM response into candidates
    fn parse_inference_response(
        &self,
        response: &str,
        context: &CommandContext,
    ) -> Result<Vec<Candidate>, AutocompleteError> {
        // Try to parse as JSON array
        let parsed: Result<Vec<InferenceCandidate>, _> = serde_json::from_str(response);

        match parsed {
            Ok(candidates) => {
                // Convert to our Candidate type
                Ok(candidates
                    .into_iter()
                    .map(|c| Candidate {
                        value: c.value,
                        description: c.description,
                        confidence: c.confidence.clamp(0.0, 1.0),
                        arg_type: self.determine_arg_type(context),
                        validated: false,
                    })
                    .collect())
            }
            Err(_) => {
                // Fallback: try to extract suggestions from text
                self.parse_fallback(response, context)
            }
        }
    }

    /// Fallback parser for non-JSON responses
    fn parse_fallback(
        &self,
        response: &str,
        context: &CommandContext,
    ) -> Result<Vec<Candidate>, AutocompleteError> {
        // Try to find JSON array in the response
        let start = response.find('[');
        let end = response.rfind(']');

        if let (Some(start), Some(end)) = (start, end) {
            let json_str = &response[start..=end];
            if let Ok(candidates) = serde_json::from_str::<Vec<InferenceCandidate>>(json_str) {
                return Ok(candidates
                    .into_iter()
                    .map(|c| Candidate {
                        value: c.value,
                        description: c.description,
                        confidence: c.confidence.clamp(0.0, 1.0),
                        arg_type: self.determine_arg_type(context),
                        validated: false,
                    })
                    .collect());
            }
        }

        // If all parsing fails, return empty list
        Ok(Vec::new())
    }

    /// Determine argument type from context
    fn determine_arg_type(&self, context: &CommandContext) -> ArgumentType {
        match context.completion_type {
            CompletionType::Command => ArgumentType::Value,
            CompletionType::Subcommand => ArgumentType::Subcommand,
            CompletionType::Flag => ArgumentType::Flag,
            CompletionType::FlagValue => ArgumentType::Option,
            CompletionType::Argument => ArgumentType::Value,
        }
    }
}

/// Intermediate type for parsing LLM response
#[derive(Debug, Deserialize)]
struct InferenceCandidate {
    value: String,
    description: String,
    confidence: f32,
}

/// Extension trait for FlagSpec
trait FlagSpecExt {
    fn matches_token(&self, token: &str) -> bool;
}

impl FlagSpecExt for super::context::FlagSpec {
    fn matches_token(&self, token: &str) -> bool {
        if let Some(short) = self.short {
            if token == format!("-{}", short) {
                return true;
            }
        }
        if let Some(long) = &self.long {
            if token == format!("--{}", long) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::backends::BackendInfo;
    use crate::models::{BackendType, GeneratedCommand, RiskLevel};

    // Mock backend for testing
    struct MockBackend;

    #[async_trait]
    impl CommandGenerator for MockBackend {
        async fn generate_command(
            &self,
            _request: &CommandRequest,
        ) -> Result<GeneratedCommand, crate::backends::GeneratorError> {
            Ok(GeneratedCommand {
                command: r#"[{"value": "commit", "description": "Record changes", "confidence": 0.9}]"#.to_string(),
                explanation: "Test".to_string(),
                safety_level: RiskLevel::Safe,
                estimated_impact: "None".to_string(),
                alternatives: vec![],
                backend_used: "mock".to_string(),
                generation_time_ms: 100,
                confidence_score: 0.9,
            })
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Mock,
                model_name: "mock".to_string(),
                supports_streaming: false,
                max_tokens: 1000,
                typical_latency_ms: 100,
                memory_usage_mb: 100,
                version: "1.0".to_string(),
            }
        }

        async fn shutdown(&self) -> Result<(), crate::backends::GeneratorError> {
            Ok(())
        }
    }

    #[test]
    fn test_inference_config_default() {
        let config = InferenceConfig::default();
        assert_eq!(config.max_candidates, 10);
        assert_eq!(config.temperature, 0.3);
        assert!(config.include_examples);
    }

    #[tokio::test]
    async fn test_parse_json_response() {
        let backend = Box::new(MockBackend);
        let agent = InferenceAgent::new(InferenceConfig::default(), backend).unwrap();

        let response = r#"[
            {"value": "commit", "description": "Record changes", "confidence": 0.9},
            {"value": "add", "description": "Add files", "confidence": 0.8}
        ]"#;

        let context = CommandContext {
            base_command: "git".to_string(),
            signature: None,
            tokens: vec!["git".to_string()],
            cursor_position: 3,
            completion_type: CompletionType::Subcommand,
        };

        let candidates = agent.parse_inference_response(response, &context).unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].value, "commit");
        assert_eq!(candidates[0].confidence, 0.9);
    }
}
