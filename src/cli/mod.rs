// CLI module - Command-line interface and user interaction

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Instant;

use crate::{
    backends::{BackendInfo, CommandGenerator, GeneratorError},
    models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel, SafetyLevel, ShellType},
    safety::SafetyValidator,
};

/// Main CLI application struct
pub struct CliApp {
    config: CliConfig,
    #[allow(dead_code)]
    backend: Box<dyn CommandGenerator>,
    validator: SafetyValidator,
}

impl std::fmt::Debug for CliApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliApp")
            .field("config", &self.config)
            .field("backend", &"<CommandGenerator>")
            .field("validator", &self.validator)
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
    fn safety(&self) -> Option<String>;
    fn output(&self) -> Option<String>;
    fn confirm(&self) -> bool;
    fn verbose(&self) -> bool;
    fn config_file(&self) -> Option<String>;
}

impl CliApp {
    /// Create new CLI application instance
    ///
    /// Uses configuration-driven backend selection with embedded model as primary
    /// and optional remote backend fallbacks.
    pub async fn new() -> Result<Self, CliError> {
        Self::with_config(CliConfig::default()).await
    }

    /// Create CLI application with custom configuration
    pub async fn with_config(config: CliConfig) -> Result<Self, CliError> {
        // Load user configuration to determine backend preferences
        let config_manager =
            crate::config::ConfigManager::new().map_err(|e| CliError::ConfigurationError {
                message: format!("Failed to create config manager: {}", e),
            })?;

        let user_config = config_manager
            .load()
            .map_err(|e| CliError::ConfigurationError {
                message: format!("Failed to load configuration: {}", e),
            })?;

        // Create backend based on configuration
        let backend = Self::create_backend(&user_config).await?;

        let validator =
            SafetyValidator::new(crate::safety::SafetyConfig::default()).map_err(|e| {
                CliError::ConfigurationError {
                    message: format!("Failed to initialize safety validator: {}", e),
                }
            })?;

        Ok(Self {
            config,
            backend,
            validator,
        })
    }

    /// Create appropriate backend based on user configuration
    async fn create_backend(
        _user_config: &crate::models::UserConfiguration,
    ) -> Result<Box<dyn CommandGenerator>, CliError> {
        // For unit tests only, use mock backend to avoid model downloads
        #[cfg(test)]
        {
            return Ok(Box::new(MockCommandGenerator::new()));
        }

        // For production, use smart backend selector with intelligent fallback
        #[cfg(not(test))]
        {
            use crate::backends::embedded::EmbeddedModelBackend;
            use crate::backends::selector::{BackendSelectorConfig, SmartBackend};
            use std::sync::Arc;

            // Create smart backend with optimized configuration
            let mut selector_config = BackendSelectorConfig::default();
            selector_config.health_check_timeout_ms = 1000; // Faster health checks for CLI
            selector_config.enable_adaptive_learning = true;

            let smart_backend = SmartBackend::new(crate::backends::selector::BackendSelector::new(
                selector_config,
            ));

            // Add embedded backend as primary (always available)
            let embedded_backend =
                EmbeddedModelBackend::new().map_err(|e| CliError::ConfigurationError {
                    message: format!("Failed to create embedded backend: {}", e),
                })?;

            smart_backend
                .add_backend(
                    Arc::new(embedded_backend),
                    "embedded-cpu".to_string(),
                    10, // Lower priority number = higher priority
                )
                .await
                .map_err(|e| CliError::ConfigurationError {
                    message: format!("Failed to add embedded backend: {}", e),
                })?;

            // Add MLX backend if on Apple Silicon
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            {
                use crate::backends::embedded::{MlxBackend, ModelVariant};
                use crate::ModelLoader;

                if ModelVariant::detect() == ModelVariant::MLX {
                    let model_loader =
                        ModelLoader::new().map_err(|e| CliError::ConfigurationError {
                            message: format!("Failed to create model loader: {}", e),
                        })?;

                    if let Ok(model_path) = model_loader.get_embedded_model_path() {
                        if let Ok(mlx_backend) = MlxBackend::new(model_path) {
                            smart_backend
                                .add_backend(
                                    Arc::new(mlx_backend),
                                    "embedded-mlx".to_string(),
                                    5, // Higher priority than CPU
                                )
                                .await
                                .map_err(|e| CliError::ConfigurationError {
                                    message: format!("Failed to add MLX backend: {}", e),
                                })?;

                            tracing::info!("Added MLX backend for Apple Silicon");
                        }
                    }
                }
            }

            // Add remote backends if feature is enabled and configured
            #[cfg(feature = "remote-backends")]
            {
                use crate::backends::remote::{OllamaBackend, VllmBackend};
                use reqwest::Url;

                // Try to add Ollama backend if configured
                if let Ok(ollama_url) = std::env::var("CMDAI_OLLAMA_URL")
                    .or_else(|_| Ok("http://localhost:11434".to_string()))
                    .and_then(|url| Url::parse(&url).map_err(|_| "invalid url"))
                {
                    let model = std::env::var("CMDAI_OLLAMA_MODEL")
                        .unwrap_or_else(|_| "qwen2.5-coder:1.5b".to_string());

                    if let Ok(ollama_backend) = OllamaBackend::new(ollama_url, model) {
                        smart_backend
                            .add_backend(
                                Arc::new(ollama_backend),
                                "ollama".to_string(),
                                2, // High priority for local servers
                            )
                            .await
                            .map_err(|e| CliError::ConfigurationError {
                                message: format!("Failed to add Ollama backend: {}", e),
                            })?;

                        tracing::info!("Added Ollama backend");
                    }
                }

                // Try to add vLLM backend if configured
                if let Ok(vllm_url) = std::env::var("CMDAI_VLLM_URL") {
                    if let Ok(url) = Url::parse(&vllm_url) {
                        let model = std::env::var("CMDAI_VLLM_MODEL")
                            .unwrap_or_else(|_| "Qwen/Qwen2.5-Coder-1.5B-Instruct".to_string());

                        if let Ok(mut vllm_backend) = VllmBackend::new(url, model) {
                            // Add API key if provided
                            if let Ok(api_key) = std::env::var("CMDAI_VLLM_API_KEY") {
                                vllm_backend = vllm_backend.with_api_key(api_key);
                            }

                            smart_backend
                                .add_backend(
                                    Arc::new(vllm_backend),
                                    "vllm".to_string(),
                                    3, // Lower priority for remote APIs
                                )
                                .await
                                .map_err(|e| CliError::ConfigurationError {
                                    message: format!("Failed to add vLLM backend: {}", e),
                                })?;

                            tracing::info!("Added vLLM backend");
                        }
                    }
                }
            }

            tracing::info!("Initialized smart backend selector with intelligent fallback");
            Ok(Box::new(smart_backend))
        }
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
        let request = CommandRequest {
            input: prompt.clone(),
            context: None,
            shell,
            safety_level,
            backend_preference: None,
        };

        // Generate command
        let gen_start = Instant::now();
        let generated = self.backend.generate_command(&request).await.map_err(|e| {
            CliError::GenerationFailed {
                details: e.to_string(),
            }
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

        // Determine if command should execute
        let executed = blocked_reason.is_none() && !requires_confirmation;

        // Build confirmation prompt
        let confirmation_prompt = if requires_confirmation {
            format!(
                "Command '{}' requires confirmation due to {} risk. Proceed? (y/N)",
                generated.command, validation.risk_level
            )
        } else {
            String::new()
        };

        // Collect debug info if verbose
        let debug_info = if args.verbose() {
            Some(format!(
                "Backend: {}, Model: {}, Confidence: {:.2}, Safety: {:?}",
                generated.backend_used, "mock-model", generated.confidence_score, safety_level
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
                execution_time_ms: 0,
                total_time_ms: total_time.as_millis() as u64,
            },
            warnings: {
                let mut all_warnings = warnings_list;
                all_warnings.extend(validation.warnings);
                all_warnings
            },
            detected_context: prompt.clone(),
        })
    }

    /// Show help information
    pub async fn show_help(&self) -> Result<String, CliError> {
        Ok(r#"cmdai - Natural language to shell command converter

USAGE:
    cmdai [OPTIONS] <PROMPT>

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
    cmdai "list all files"
    cmdai --shell zsh "find large files"
    cmdai --safety strict "delete temporary files"
"#
        .to_string())
    }

    /// Show version information
    pub async fn show_version(&self) -> Result<String, CliError> {
        Ok(format!("cmdai v{}", env!("CARGO_PKG_VERSION")))
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
struct MockCommandGenerator;

#[cfg(any(test, debug_assertions))]
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
