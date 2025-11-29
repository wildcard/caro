//! Integration tests for autocomplete inference and validation
//!
//! These tests verify the end-to-end functionality of the autocomplete system:
//! - Context parsing for partial commands
//! - LLM-based inference of completions
//! - Validation of suggested arguments

use cmdai::autocomplete::{
    ArgumentSpec, ArgumentValidator, AutocompleteConfig, AutocompleteEngine, CommandSignature,
    CompletionContext, FlagSpec, InferenceAgent, InferenceConfig, SubcommandSpec, ValidatorConfig,
};
use cmdai::backends::{BackendInfo, CommandGenerator, GeneratorError};
use cmdai::models::{
    BackendType, CommandRequest, GeneratedCommand, RiskLevel, ShellType,
};

/// Mock backend for testing that returns predefined responses
struct MockAutocompleteBackend {
    response: String,
}

impl MockAutocompleteBackend {
    fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }

    fn git_commit_response() -> Self {
        Self::new(
            r#"[
                {"value": "-m", "description": "Commit message", "confidence": 0.95},
                {"value": "-a", "description": "Stage all changes", "confidence": 0.85},
                {"value": "--amend", "description": "Amend previous commit", "confidence": 0.75}
            ]"#,
        )
    }

    fn file_path_response() -> Self {
        Self::new(
            r#"[
                {"value": "README.md", "description": "Project readme", "confidence": 0.9},
                {"value": "Cargo.toml", "description": "Package manifest", "confidence": 0.85},
                {"value": "src/", "description": "Source directory", "confidence": 0.8}
            ]"#,
        )
    }
}

#[async_trait::async_trait]
impl CommandGenerator for MockAutocompleteBackend {
    async fn generate_command(
        &self,
        _request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        Ok(GeneratedCommand {
            command: self.response.clone(),
            explanation: "Mock autocomplete response".to_string(),
            safety_level: RiskLevel::Safe,
            estimated_impact: "None".to_string(),
            alternatives: vec![],
            backend_used: "mock".to_string(),
            generation_time_ms: 10,
            confidence_score: 0.9,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Mock,
            model_name: "mock-autocomplete".to_string(),
            supports_streaming: false,
            max_tokens: 1000,
            typical_latency_ms: 10,
            memory_usage_mb: 10,
            version: "1.0.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_completion_context_parsing() {
    let context = CompletionContext::new().unwrap();

    // Test git commit command parsing
    let cmd_context = context
        .get_context_for_command("git commit ", 11)
        .unwrap();

    assert_eq!(cmd_context.base_command, "git");
    assert!(cmd_context.signature.is_some());

    let signature = cmd_context.signature.unwrap();
    assert_eq!(signature.command, "git");
    assert!(!signature.subcommands.is_empty());

    // Verify git commit subcommand exists
    let commit_subcommand = signature
        .subcommands
        .iter()
        .find(|s| s.name == "commit");
    assert!(commit_subcommand.is_some());
}

#[tokio::test]
async fn test_inference_with_mock_backend() {
    let backend = Box::new(MockAutocompleteBackend::git_commit_response());
    let agent = InferenceAgent::new(InferenceConfig::default(), backend).unwrap();

    let context = CompletionContext::new().unwrap();
    let cmd_context = context
        .get_context_for_command("git commit ", 11)
        .unwrap();

    let candidates = agent
        .infer_completions("git commit ", 11, &cmd_context)
        .await
        .unwrap();

    assert!(!candidates.is_empty());
    assert!(candidates.iter().any(|c| c.value == "-m"));
    assert!(candidates.iter().any(|c| c.value == "-a"));

    // Verify confidence scores are valid
    for candidate in &candidates {
        assert!(candidate.confidence >= 0.0 && candidate.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_argument_validation_string_pattern() {
    let validator = ArgumentValidator::new(ValidatorConfig::default()).unwrap();

    let spec = ArgumentSpec::String {
        pattern: Some(r"^v\d+\.\d+\.\d+$".to_string()),
        examples: vec!["v1.0.0".to_string(), "v2.3.4".to_string()],
    };

    // Valid version string
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validator
        .validate_against_spec("v1.2.3", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(errors.is_empty(), "Expected v1.2.3 to be valid");

    // Invalid version string
    let mut errors = Vec::new();
    validator
        .validate_against_spec("1.2.3", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(!errors.is_empty(), "Expected 1.2.3 to be invalid");
}

#[tokio::test]
async fn test_argument_validation_enum() {
    let validator = ArgumentValidator::new(ValidatorConfig::default()).unwrap();

    let spec = ArgumentSpec::Enum {
        values: vec![
            "debug".to_string(),
            "info".to_string(),
            "warn".to_string(),
            "error".to_string(),
        ],
    };

    // Valid enum value
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validator
        .validate_against_spec("info", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(errors.is_empty());

    // Invalid enum value
    let mut errors = Vec::new();
    validator
        .validate_against_spec("trace", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(!errors.is_empty());
}

#[tokio::test]
async fn test_argument_validation_integer_range() {
    let validator = ArgumentValidator::new(ValidatorConfig::default()).unwrap();

    let spec = ArgumentSpec::Integer {
        min: Some(1),
        max: Some(100),
    };

    // Valid integer
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validator
        .validate_against_spec("50", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(errors.is_empty());

    // Below minimum
    let mut errors = Vec::new();
    validator
        .validate_against_spec("0", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(!errors.is_empty());

    // Above maximum
    let mut errors = Vec::new();
    validator
        .validate_against_spec("101", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(!errors.is_empty());

    // Not an integer
    let mut errors = Vec::new();
    validator
        .validate_against_spec("abc", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(!errors.is_empty());
}

#[tokio::test]
async fn test_end_to_end_autocomplete_engine() {
    let backend = Box::new(MockAutocompleteBackend::git_commit_response());
    let config = AutocompleteConfig {
        max_suggestions: 5,
        min_confidence: 0.7,
        enable_validation: true,
        shell_type: ShellType::Bash,
    };

    let engine = AutocompleteEngine::new(config, backend).unwrap();

    let result = engine.suggest("git commit ", 11).await.unwrap();

    assert_eq!(result.partial_command, "git commit ");
    assert_eq!(result.cursor_position, 11);
    assert!(!result.candidates.is_empty());

    // Verify all candidates meet minimum confidence threshold
    for candidate in &result.candidates {
        assert!(candidate.confidence >= 0.7);
    }

    // Verify suggestions are sorted by confidence
    for i in 1..result.candidates.len() {
        assert!(
            result.candidates[i - 1].confidence >= result.candidates[i].confidence,
            "Candidates should be sorted by confidence (descending)"
        );
    }
}

#[tokio::test]
async fn test_custom_command_signature() {
    let backend = Box::new(MockAutocompleteBackend::file_path_response());
    let config = AutocompleteConfig::default();

    let mut engine = AutocompleteEngine::new(config, backend).unwrap();

    // Add custom command signature
    let custom_command = CommandSignature {
        command: "deploy".to_string(),
        description: "Deploy application".to_string(),
        subcommands: vec![SubcommandSpec {
            name: "production".to_string(),
            description: "Deploy to production".to_string(),
            flags: vec![FlagSpec {
                short: Some('v'),
                long: Some("version".to_string()),
                description: "Version to deploy".to_string(),
                takes_value: true,
                value_spec: Some(ArgumentSpec::String {
                    pattern: Some(r"^v\d+\.\d+\.\d+$".to_string()),
                    examples: vec!["v1.0.0".to_string()],
                }),
            }],
            arguments: vec![],
        }],
        global_flags: vec![],
    };

    engine.add_command_signature(custom_command);

    // Test that custom command is recognized
    let result = engine.suggest("deploy production ", 18).await.unwrap();
    assert_eq!(result.partial_command, "deploy production ");
}

#[tokio::test]
async fn test_validation_with_file_spec() {
    let validator = ArgumentValidator::new(ValidatorConfig {
        check_file_existence: false, // Don't check actual filesystem
        check_directory_existence: false,
        strict_pattern_matching: false,
    })
    .unwrap();

    let spec = ArgumentSpec::File {
        must_exist: false,
        extensions: Some(vec!["rs".to_string(), "toml".to_string()]),
    };

    // Valid file with correct extension
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validator
        .validate_against_spec("main.rs", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(errors.is_empty());

    // File with wrong extension
    let mut errors = Vec::new();
    validator
        .validate_against_spec("main.py", &spec, &mut errors, &mut warnings)
        .await
        .unwrap();
    assert!(!errors.is_empty());
}

#[tokio::test]
async fn test_confidence_filtering() {
    let backend = Box::new(MockAutocompleteBackend::git_commit_response());
    let config = AutocompleteConfig {
        max_suggestions: 10,
        min_confidence: 0.8, // Higher threshold
        enable_validation: false,
        shell_type: ShellType::Bash,
    };

    let engine = AutocompleteEngine::new(config, backend).unwrap();
    let result = engine.suggest("git commit ", 11).await.unwrap();

    // All candidates should have confidence >= 0.8
    for candidate in &result.candidates {
        assert!(
            candidate.confidence >= 0.8,
            "Candidate {} has confidence {} which is below threshold 0.8",
            candidate.value,
            candidate.confidence
        );
    }
}

#[tokio::test]
async fn test_max_suggestions_limit() {
    let backend = Box::new(MockAutocompleteBackend::git_commit_response());
    let config = AutocompleteConfig {
        max_suggestions: 2,
        min_confidence: 0.0,
        enable_validation: false,
        shell_type: ShellType::Bash,
    };

    let engine = AutocompleteEngine::new(config, backend).unwrap();
    let result = engine.suggest("git commit ", 11).await.unwrap();

    assert!(
        result.candidates.len() <= 2,
        "Expected at most 2 candidates, got {}",
        result.candidates.len()
    );
}
