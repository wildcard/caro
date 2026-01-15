// CLI module - Command-line interface and user interaction

pub mod telemetry;

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use crate::{
    agent::AgentLoop,
    backends::CommandGenerator,
    context::ExecutionContext,
    models::{CommandRequest, SafetyLevel, ShellType},
    prompts::CapabilityProfile,
    safety::SafetyValidator,
};

#[cfg(any(test, debug_assertions))]
use async_trait::async_trait;

#[cfg(any(test, debug_assertions))]
use crate::{
    backends::{BackendInfo, GeneratorError},
    models::{BackendType, GeneratedCommand, RiskLevel},
};

/// Main CLI application struct
pub struct CliApp {
    config: CliConfig,
    #[allow(dead_code)]
    backend: Arc<dyn CommandGenerator>,
    agent_loop: AgentLoop,
    validator: SafetyValidator,
    #[allow(dead_code)]
    context: ExecutionContext,
}

impl std::fmt::Debug for CliApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliApp")
            .field("config", &self.config)
            .field("backend", &"<CommandGenerator>")
            .field("validator", &self.validator)
            .field("context", &"<ExecutionContext>")
            .finish()
    }
}

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub default_shell: ShellType,
    pub safety_level: SafetyLevel,
    pub output_format: OutputFormat,
    pub auto_confirm: bool,
}

/// Result of CLI command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResult {
    pub generated_command: String,
    pub explanation: String,
    pub executed: bool,
    pub blocked_reason: Option<String>,
    pub requires_confirmation: bool,
    pub confirmation_prompt: String,
    pub alternatives: Vec<String>,
    pub shell_used: ShellType,
    pub output_format: OutputFormat,
    pub debug_info: Option<String>,
    pub generation_details: String,
    pub timing_info: TimingInfo,
    pub warnings: Vec<String>,
    pub detected_context: String,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub execution_error: Option<String>,
}

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Yaml,
    Plain,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "plain" => Ok(OutputFormat::Plain),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Timing information for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimingInfo {
    pub generation_time_ms: u64,
    pub execution_time_ms: u64,
    pub total_time_ms: u64,
}

/// Parsed CLI arguments
#[derive(Debug, Clone)]
pub struct ParsedArgs {
    pub prompt: Option<String>,
    pub shell: Option<String>,
    pub safety: Option<String>,
    pub output: Option<String>,
    pub confirm: bool,
    pub verbose: bool,
    pub config_file: Option<String>,
}

/// Trait for types that can be converted to CLI arguments
pub trait IntoCliArgs {
    fn prompt(&self) -> Option<String>;
    fn shell(&self) -> Option<String>;
    fn backend(&self) -> Option<String>;
    fn model_name(&self) -> Option<String>;
    fn safety(&self) -> Option<String>;
    fn output(&self) -> Option<String>;
    fn confirm(&self) -> bool;
    fn verbose(&self) -> bool;
    fn config_file(&self) -> Option<String>;
    fn execute(&self) -> bool;
    fn dry_run(&self) -> bool;
    fn interactive(&self) -> bool;
    fn force_llm(&self) -> bool;
}

impl CliApp {
    /// Create new CLI application instance
    ///
    /// Uses configuration-driven backend selection with embedded model as primary
    /// and optional remote backend fallbacks.
    pub async fn new() -> Result<Self, CliError> {
        Self::with_overrides(CliConfig::default(), None, None, false).await
    }

    /// Create CLI application with backend and model overrides from CLI args
    ///
    /// Backend selection priority (highest to lowest):
    /// 1. CLI flag (`--backend`)
    /// 2. Environment variable (`CARO_BACKEND`)
    /// 3. Config file (`~/.config/caro/config.toml`)
    /// 4. Auto-detect (default: embedded)
    pub async fn with_overrides(
        config: CliConfig,
        backend_override: Option<String>,
        model_name_override: Option<String>,
        force_llm: bool,
    ) -> Result<Self, CliError> {
        // Load user configuration to determine backend preferences
        let config_manager =
            crate::config::ConfigManager::new().map_err(|e| CliError::ConfigurationError {
                message: format!("Failed to create config manager: {}", e),
            })?;

        let mut user_config = config_manager
            .load()
            .map_err(|e| CliError::ConfigurationError {
                message: format!("Failed to load configuration: {}", e),
            })?;

        // Backend selection priority: CLI flag > env var > config file
        let env_backend = std::env::var("CARO_BACKEND").ok();
        let backend_source = if backend_override.is_some() {
            "CLI flag"
        } else if env_backend.is_some() {
            "CARO_BACKEND env"
        } else if user_config.default_model.is_some() {
            "config file"
        } else {
            "auto-detect"
        };

        let effective_backend = backend_override
            .or(env_backend)
            .or_else(|| user_config.default_model.clone());

        // Validate backend name if specified
        if let Some(ref backend) = effective_backend {
            Self::validate_backend_name(backend)?;
            user_config.default_model = Some(backend.clone());
            tracing::debug!(
                "Backend preference: {} (source: {})",
                backend,
                backend_source
            );
        }

        // Model name: CLI flag > env var > config file
        let effective_model_name = model_name_override
            .or_else(|| std::env::var("CARO_MODEL").ok())
            .or_else(|| user_config.model_name.clone());

        if let Some(model_name) = effective_model_name {
            user_config.model_name = Some(model_name);
        }

        // Create backend based on configuration
        let backend = Self::create_backend(&user_config).await?;
        let backend_arc: Arc<dyn CommandGenerator> = Arc::from(backend);

        let validator =
            SafetyValidator::new(crate::safety::SafetyConfig::default()).map_err(|e| {
                CliError::ConfigurationError {
                    message: format!("Failed to initialize safety validator: {}", e),
                }
            })?;

        // Detect execution context
        let context = ExecutionContext::detect();

        // Detect platform capabilities for command generation (uses cache for fast startup)
        let profile = CapabilityProfile::detect_or_cached().await;

        // Create agent loop with backend, context, and profile
        // If force_llm is true, disable the static matcher
        let agent_loop = AgentLoop::new(backend_arc.clone(), context.clone(), profile)
            .with_static_matcher(!force_llm);

        Ok(Self {
            config,
            backend: backend_arc,
            agent_loop,
            validator,
            context,
        })
    }

    /// Create appropriate backend based on user configuration
    async fn create_backend(
        user_config: &crate::models::UserConfiguration,
    ) -> Result<Box<dyn CommandGenerator>, CliError> {
        // For test builds only, use mock backend
        #[cfg(test)]
        {
            let _ = user_config; // Suppress unused warning in test builds
            Ok(Box::new(MockCommandGenerator::new()))
        }

        // Real backend selection (debug and release builds)
        #[cfg(not(test))]
        {
            // Allow explicit mock backend via environment variable for testing
            #[cfg(feature = "mock-backend")]
            if std::env::var("CARO_MOCK_BACKEND").is_ok() {
                tracing::info!("Using mock backend (CARO_MOCK_BACKEND set)");
                return Ok(Box::new(MockCommandGenerator::new()));
            }

            use crate::backends::embedded::EmbeddedModelBackend;
            use std::sync::Arc;

            // Create embedded backend (used as fallback or primary)
            let embedded_backend =
                EmbeddedModelBackend::new().map_err(|e| CliError::ConfigurationError {
                    message: format!("Failed to create embedded backend: {}", e),
                })?;

            let embedded_arc: Arc<EmbeddedModelBackend> = Arc::new(embedded_backend);

            // Check for user-specified model preference
            let model_preference = user_config.default_model.as_deref();

            // If user explicitly specified a model, try that first
            if let Some(model) = model_preference {
                tracing::info!("User requested backend: {}", model);

                match model {
                    "embedded" => {
                        tracing::info!("Using embedded backend (user preference)");
                        return match std::sync::Arc::try_unwrap(embedded_arc) {
                            Ok(backend) => Ok(Box::new(backend)),
                            Err(arc) => Ok(Box::new((*arc).clone())),
                        };
                    }
                    #[cfg(feature = "remote-backends")]
                    "ollama" => {
                        use crate::backends::remote::OllamaBackend;
                        use reqwest::Url;

                        if let Ok(ollama_url) = Url::parse("http://localhost:11434") {
                            let ollama_backend =
                                OllamaBackend::new(ollama_url, "codellama:7b".to_string())
                                    .map_err(|e| CliError::ConfigurationError {
                                        message: format!("Failed to create Ollama backend: {}", e),
                                    })?
                                    .with_embedded_fallback(embedded_arc.clone());

                            if ollama_backend.is_available().await {
                                tracing::info!("Using Ollama backend (user preference)");
                                return Ok(Box::new(ollama_backend));
                            } else {
                                tracing::warn!(
                                    "Ollama backend not available, falling back to embedded"
                                );
                            }
                        }
                    }
                    #[cfg(feature = "remote-backends")]
                    "exo" => {
                        use crate::backends::remote::ExoBackend;
                        use reqwest::Url;

                        if let Ok(exo_url) = Url::parse("http://localhost:52415") {
                            let exo_backend = ExoBackend::new(exo_url, "llama-3.2-3b".to_string())
                                .map_err(|e| CliError::ConfigurationError {
                                    message: format!("Failed to create Exo backend: {}", e),
                                })?
                                .with_embedded_fallback(embedded_arc.clone());

                            if exo_backend.is_available().await {
                                tracing::info!("Using Exo backend (user preference)");
                                return Ok(Box::new(exo_backend));
                            } else {
                                tracing::warn!(
                                    "Exo backend not available, falling back to embedded"
                                );
                            }
                        }
                    }
                    #[cfg(feature = "remote-backends")]
                    "vllm" => {
                        use crate::backends::remote::VllmBackend;
                        use reqwest::Url;

                        if let Ok(vllm_url) = Url::parse("http://localhost:8000") {
                            let vllm_backend =
                                VllmBackend::new(vllm_url, "codellama/CodeLlama-7b-hf".to_string())
                                    .map_err(|e| CliError::ConfigurationError {
                                        message: format!("Failed to create vLLM backend: {}", e),
                                    })?
                                    .with_embedded_fallback(embedded_arc.clone());

                            if vllm_backend.is_available().await {
                                tracing::info!("Using vLLM backend (user preference)");
                                return Ok(Box::new(vllm_backend));
                            } else {
                                tracing::warn!(
                                    "vLLM backend not available, falling back to embedded"
                                );
                            }
                        }
                    }
                    #[cfg(not(feature = "remote-backends"))]
                    "ollama" | "exo" | "vllm" => {
                        tracing::warn!(
                            "Remote backends not compiled in. Build with --features remote-backends"
                        );
                    }
                    _ => {
                        tracing::warn!("Unknown backend '{}', using auto-detect", model);
                    }
                }
            }

            // Auto-detect: try remote backends with embedded fallback
            #[cfg(feature = "remote-backends")]
            {
                use crate::backends::remote::{ExoBackend, OllamaBackend, VllmBackend};
                use reqwest::Url;

                // Priority: Exo cluster > Ollama > vLLM > Embedded
                if let Ok(exo_url) = Url::parse("http://localhost:52415") {
                    let exo_backend = ExoBackend::new(exo_url, "llama-3.2-3b".to_string())
                        .map_err(|e| CliError::ConfigurationError {
                            message: format!("Failed to create Exo backend: {}", e),
                        })?
                        .with_embedded_fallback(embedded_arc.clone());

                    if exo_backend.is_available().await {
                        tracing::info!("Using Exo cluster backend (auto-detected)");
                        return Ok(Box::new(exo_backend));
                    }
                }

                if let Ok(ollama_url) = Url::parse("http://localhost:11434") {
                    let ollama_backend = OllamaBackend::new(ollama_url, "codellama:7b".to_string())
                        .map_err(|e| CliError::ConfigurationError {
                            message: format!("Failed to create Ollama backend: {}", e),
                        })?
                        .with_embedded_fallback(embedded_arc.clone());

                    if ollama_backend.is_available().await {
                        tracing::info!("Using Ollama backend (auto-detected)");
                        return Ok(Box::new(ollama_backend));
                    }
                }

                if let Ok(vllm_url) = Url::parse("http://localhost:8000") {
                    let vllm_backend =
                        VllmBackend::new(vllm_url, "codellama/CodeLlama-7b-hf".to_string())
                            .map_err(|e| CliError::ConfigurationError {
                                message: format!("Failed to create vLLM backend: {}", e),
                            })?
                            .with_embedded_fallback(embedded_arc.clone());

                    if vllm_backend.is_available().await {
                        tracing::info!("Using vLLM backend (auto-detected)");
                        return Ok(Box::new(vllm_backend));
                    }
                }
            }

            // Fall back to embedded backend only
            tracing::info!("Using embedded backend only");
            match std::sync::Arc::try_unwrap(embedded_arc) {
                Ok(backend) => Ok(Box::new(backend)),
                Err(arc) => Ok(Box::new((*arc).clone())),
            }
        }
    }

    /// Validate that the backend name is valid
    ///
    /// Returns Ok(()) if valid, or a helpful error message if not.
    fn validate_backend_name(backend: &str) -> Result<(), CliError> {
        const VALID_BACKENDS: &[&str] = &["embedded", "ollama", "exo", "vllm"];

        let normalized = backend.to_lowercase();
        if VALID_BACKENDS.contains(&normalized.as_str()) {
            return Ok(());
        }

        // Provide helpful error with suggestions
        let suggestion = VALID_BACKENDS
            .iter()
            .find(|&&v| v.starts_with(&normalized) || normalized.starts_with(v))
            .map(|&v| format!(". Did you mean '{}'?", v))
            .unwrap_or_default();

        Err(CliError::InvalidArgument {
            message: format!(
                "Unknown backend '{}'{}\n\nAvailable backends:\n  \
                 - embedded: Local Qwen model (default, no setup required)\n  \
                 - ollama: Ollama server (requires: ollama serve)\n  \
                 - exo: Exo distributed cluster (requires: exo cluster)\n  \
                 - vllm: vLLM HTTP API (requires: vllm server)\n\n\
                 Set via: --backend <name>, CARO_BACKEND env var, or config file",
                backend, suggestion
            ),
        })
    }

    /// Get list of available backend names
    pub fn available_backends() -> &'static [&'static str] {
        &["embedded", "ollama", "exo", "vllm"]
    }

    /// Run CLI with provided arguments
    pub async fn run_with_args<T>(&self, args: T) -> Result<CliResult, CliError>
    where
        T: IntoCliArgs,
    {
        let start_time = Instant::now();
        let mut warnings_list = Vec::new();

        // Parse shell type
        let shell = if let Some(shell_str) = args.shell() {
            let parsed = ShellType::from_str(&shell_str).unwrap_or(self.config.default_shell);
            if matches!(parsed, ShellType::Unknown) {
                warnings_list.push(format!(
                    "Invalid shell '{}', using default {}",
                    shell_str, self.config.default_shell
                ));
                self.config.default_shell
            } else {
                parsed
            }
        } else {
            self.config.default_shell
        };

        // Parse safety level
        let safety_level = if let Some(safety_str) = args.safety() {
            SafetyLevel::from_str(&safety_str).unwrap_or(self.config.safety_level)
        } else {
            self.config.safety_level
        };

        // Parse output format
        let output_format = if let Some(output_str) = args.output() {
            OutputFormat::from_str(&output_str).unwrap_or(self.config.output_format)
        } else {
            self.config.output_format
        };

        // Get the prompt
        let prompt = args.prompt().ok_or_else(|| CliError::InvalidArgument {
            message: "No prompt provided".to_string(),
        })?;

        // Create command request
        let _request = CommandRequest {
            input: prompt.clone(),
            context: None,
            shell,
            safety_level,
            backend_preference: None,
        };

        // Generate command using agent loop (handles iterations internally)
        let gen_start = Instant::now();
        let generated = self
            .agent_loop
            .generate_command(&prompt)
            .await
            .map_err(|e| CliError::GenerationFailed {
                details: e.to_string(),
            })?;
        let generation_time = gen_start.elapsed();

        // Validate command safety
        let validation = self
            .validator
            .validate_command(&generated.command, shell)
            .await
            .map_err(|e| CliError::Internal {
                message: format!("Safety validation failed: {}", e),
            })?;

        // Check if confirmation is required
        let requires_confirmation =
            validation.risk_level.requires_confirmation(safety_level) && !args.confirm();

        let blocked_reason = if validation.risk_level.is_blocked(safety_level) {
            Some(format!(
                "Command blocked due to {} risk: {}",
                validation.risk_level,
                validation.warnings.join(", ")
            ))
        } else {
            None
        };

        // Determine if command passes safety checks
        let can_execute = blocked_reason.is_none() && !requires_confirmation;

        // Build confirmation prompt
        let confirmation_prompt = if requires_confirmation {
            format!(
                "Command '{}' requires confirmation due to {} risk. Proceed? (y/N)",
                generated.command, validation.risk_level
            )
        } else {
            String::new()
        };

        // Execute command if requested and allowed
        // Note: dry_run prevents execution even if execute/interactive flags are set
        let (exit_code, stdout, stderr, execution_error, execution_time_ms) =
            if (args.execute() || args.interactive()) && can_execute && !args.dry_run() {
                use crate::execution::CommandExecutor;

                let executor = CommandExecutor::new(shell);
                match executor.execute(&generated.command) {
                    Ok(result) => (
                        Some(result.exit_code),
                        Some(result.stdout),
                        Some(result.stderr),
                        if !result.success {
                            Some(format!("Command exited with code {}", result.exit_code))
                        } else {
                            None
                        },
                        result.execution_time_ms,
                    ),
                    Err(e) => (
                        None,
                        None,
                        None,
                        Some(format!("Execution failed: {}", e)),
                        0,
                    ),
                }
            } else {
                (None, None, None, None, 0)
            };

        // The 'executed' field indicates whether safety checks passed (original behavior)
        let executed = can_execute;

        // Collect debug info if verbose
        let debug_info = if args.verbose() {
            let backend_info = self.backend.backend_info();
            Some(format!(
                "Backend: {}, Model: {}, Confidence: {:.2}, Safety: {:?}",
                generated.backend_used,
                backend_info.model_name,
                generated.confidence_score,
                safety_level
            ))
        } else {
            None
        };

        let total_time = start_time.elapsed();

        Ok(CliResult {
            generated_command: generated.command,
            explanation: generated.explanation,
            executed,
            blocked_reason,
            requires_confirmation,
            confirmation_prompt,
            alternatives: generated.alternatives,
            shell_used: shell,
            output_format,
            debug_info,
            generation_details: if args.verbose() {
                format!(
                    "Generated in {}ms using {} backend",
                    generation_time.as_millis(),
                    generated.backend_used
                )
            } else {
                String::new()
            },
            timing_info: TimingInfo {
                generation_time_ms: generation_time.as_millis() as u64,
                execution_time_ms,
                total_time_ms: total_time.as_millis() as u64,
            },
            warnings: {
                let mut all_warnings = warnings_list;
                all_warnings.extend(validation.warnings);
                all_warnings
            },
            detected_context: prompt.clone(),
            exit_code,
            stdout,
            stderr,
            execution_error,
        })
    }

    /// Show help information
    pub async fn show_help(&self) -> Result<String, CliError> {
        Ok(r#"caro - Natural language to shell command converter

USAGE:
    caro [OPTIONS] <PROMPT>

OPTIONS:
    -s, --shell <SHELL>       Shell type (bash, zsh, fish, sh, powershell, cmd)
    --safety <LEVEL>          Safety level (strict, moderate, permissive)
    -o, --output <FORMAT>     Output format (json, yaml, plain)
    -y, --confirm             Auto-confirm dangerous commands
    -v, --verbose             Verbose output with debug info
    -c, --config <FILE>       Configuration file path
    -h, --help                Show this help message
    -V, --version             Show version information

EXAMPLES:
    caro "list all files"
    caro --shell zsh "find large files"
    caro --safety strict "delete temporary files"
"#
        .to_string())
    }

    /// Show version information
    ///
    /// # Arguments
    /// * `verbose` - If true, show detailed build information with Caro's personality
    pub async fn show_version(&self, verbose: bool) -> Result<String, CliError> {
        let info = crate::version::info();
        Ok(if verbose { info.long() } else { info.short() })
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            output_format: OutputFormat::Plain,
            auto_confirm: false,
        }
    }
}

/// Errors that can occur during CLI operations
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum CliError {
    #[error("CLI functionality not implemented yet")]
    NotImplemented,

    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Command generation failed: {details}")]
    GenerationFailed { details: String },

    #[error("Command execution failed: {details}")]
    ExecutionFailed { details: String },

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Internal CLI error: {message}")]
    Internal { message: String },
}

/// Mock command generator for testing
///
/// SECURITY: This mock is restricted to debug builds via #[cfg(any(test, debug_assertions))].
/// It can generate dangerous commands for testing the safety validator,
/// which would be a security risk if used in production.
/// Production (release) builds will not include this code.
#[cfg(any(test, debug_assertions))]
#[allow(dead_code)]
struct MockCommandGenerator;

#[cfg(any(test, debug_assertions))]
#[allow(dead_code)]
impl MockCommandGenerator {
    fn new() -> Self {
        Self
    }
}

#[cfg(any(test, debug_assertions))]
#[async_trait]
impl CommandGenerator for MockCommandGenerator {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        use std::time::Duration;

        // Simulate generation time
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Analyze the input to determine appropriate command
        let command = if request.input.contains("list") && request.input.contains("files") {
            match request.shell {
                ShellType::PowerShell => "Get-ChildItem".to_string(),
                ShellType::Cmd => "dir".to_string(),
                _ => "ls -la".to_string(),
            }
        } else if request.input.contains("directory") || request.input.contains("pwd") {
            "pwd".to_string()
        } else if request.input.contains("delete") && request.input.contains("system") {
            // Very dangerous command for testing
            "rm -rf /".to_string()
        } else if request.input.contains("delete") || request.input.contains("remove") {
            "rm -rf /tmp/*".to_string() // Potentially dangerous
        } else {
            format!("echo '{}'", request.input)
        };

        Ok(GeneratedCommand {
            command,
            explanation: format!("Command for: {}", request.input),
            safety_level: RiskLevel::Safe,
            estimated_impact: Default::default(),
            alternatives: vec!["Alternative command".to_string()],
            backend_used: "mock".to_string(),
            generation_time_ms: 50,
            confidence_score: 0.95,
        })
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Ollama,
            model_name: "mock-model".to_string(),
            supports_streaming: false,
            max_tokens: 1000,
            typical_latency_ms: 50,
            memory_usage_mb: 100,
            version: "1.0.0".to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

// Types are already public, no re-export needed

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_backend_name_valid() {
        assert!(CliApp::validate_backend_name("embedded").is_ok());
        assert!(CliApp::validate_backend_name("ollama").is_ok());
        assert!(CliApp::validate_backend_name("exo").is_ok());
        assert!(CliApp::validate_backend_name("vllm").is_ok());
    }

    #[test]
    fn test_validate_backend_name_case_insensitive() {
        assert!(CliApp::validate_backend_name("EMBEDDED").is_ok());
        assert!(CliApp::validate_backend_name("Ollama").is_ok());
        assert!(CliApp::validate_backend_name("EXO").is_ok());
        assert!(CliApp::validate_backend_name("VLLM").is_ok());
    }

    #[test]
    fn test_validate_backend_name_invalid() {
        let result = CliApp::validate_backend_name("unknown");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown backend 'unknown'"));
        assert!(err.to_string().contains("Available backends:"));
    }

    #[test]
    fn test_validate_backend_name_suggestion() {
        // Should suggest "ollama" for "olla"
        let result = CliApp::validate_backend_name("olla");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Did you mean 'ollama'?"));
    }

    #[test]
    fn test_available_backends() {
        let backends = CliApp::available_backends();
        assert!(backends.contains(&"embedded"));
        assert!(backends.contains(&"ollama"));
        assert!(backends.contains(&"exo"));
        assert!(backends.contains(&"vllm"));
    }
}
