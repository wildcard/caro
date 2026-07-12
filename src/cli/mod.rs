// CLI module - Command-line interface and user interaction

pub mod edit_prompt;
pub mod telemetry;

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use crate::{
    agent::AgentLoop,
    backends::CommandGenerator,
    context::ExecutionContext,
    models::{ApprovalMode, CommandRequest, RiskJudgeContext, SafetyLevel, ShellType},
    prompts::CapabilityProfile,
    safety::SafetyValidator,
};

#[cfg(not(test))]
use crate::safety::SafetyConfig;

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

impl CliApp {
    /// Clone the configured backend Arc for reuse by the AI REPL / once-mode.
    pub fn backend_arc(&self) -> Arc<dyn CommandGenerator> {
        Arc::clone(&self.backend)
    }
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
    /// How accept/prompt/block decisions are made (resolved from config file;
    /// overridden per-invocation by `--approval` / `CARO_APPROVAL`).
    #[serde(default)]
    pub approval_mode: ApprovalMode,
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
    /// Whether explanation mode is enabled
    #[serde(default)]
    pub explain_mode: bool,
    /// Detailed explanation for explain mode
    #[serde(default)]
    pub detailed_explanation: Option<crate::prompts::CommandExplanation>,
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
    /// Approval mode (`--approval prompt|auto|smart`).
    /// Default returns `None` (resolve to config/default) for back-compat.
    fn approval(&self) -> Option<String> {
        None
    }
    fn verbose(&self) -> bool;
    fn config_file(&self) -> Option<String>;
    fn execute(&self) -> bool;
    fn dry_run(&self) -> bool;
    fn interactive(&self) -> bool;
    fn force_llm(&self) -> bool;
    fn explain(&self) -> bool;
    /// Suppress timing / progress output (`--quiet`).
    /// Default implementation returns `false` for backward compatibility.
    fn quiet(&self) -> bool {
        false
    }
    /// Disable telemetry for this invocation only (`--no-telemetry`).
    /// Default implementation returns `false` for backward compatibility.
    fn no_telemetry(&self) -> bool {
        false
    }
    /// Print backend info and exit (`--backend-info`).
    /// Default implementation returns `false` for backward compatibility.
    fn backend_info(&self) -> bool {
        false
    }
}

impl CliApp {
    /// Create new CLI application instance
    ///
    /// Uses configuration-driven backend selection with embedded model as primary
    /// and optional remote backend fallbacks.
    pub async fn new() -> Result<Self, CliError> {
        Self::with_overrides(CliConfig::default(), None, None, false, None).await
    }

    /// Create CLI application with backend and model overrides from CLI args
    ///
    /// Backend selection priority (highest to lowest):
    /// 1. CLI flag (`--backend`)
    /// 2. Environment variable (`CARO_BACKEND`)
    /// 3. Config file (`~/.config/caro/config.toml`)
    /// 4. Auto-detect (default: embedded)
    pub async fn with_overrides(
        mut config: CliConfig,
        backend_override: Option<String>,
        model_name_override: Option<String>,
        force_llm: bool,
        advisor_override: Option<String>,
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

        // Carry the config-file approval mode into the runtime config; the
        // per-invocation `--approval` flag / `CARO_APPROVAL` env override it
        // later in `run_with_args`.
        config.approval_mode = user_config.approval_mode;

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

        // Build a safety validator that honors both the user's safety level
        // AND their custom patterns / allowlist from config.toml + sibling
        // patterns.toml. Critical built-ins still take precedence over any
        // user allowlist (see `SafetyValidator::validate_command`).
        let safety_config = crate::safety::SafetyConfig::from_user_config(
            &user_config,
            config_manager.config_path(),
        );
        let validator =
            SafetyValidator::new(safety_config).map_err(|e| CliError::ConfigurationError {
                message: format!("Failed to initialize safety validator: {}", e),
            })?;

        // Detect execution context
        let context = ExecutionContext::detect();

        // Detect platform capabilities for command generation (uses cache for fast startup)
        let profile = CapabilityProfile::detect_or_cached().await;

        // Create agent loop with backend, context, and profile
        // If force_llm is true, disable the static matcher
        #[allow(unused_mut)]
        let mut agent_loop = AgentLoop::new(backend_arc.clone(), context.clone(), profile)
            .with_static_matcher(!force_llm);

        // Optional frontier advisor (off by default). Consulted only on
        // low-confidence drafts; its output is re-validated before use.
        if let Some(advisor_name) = advisor_override.as_deref() {
            agent_loop = Self::maybe_attach_advisor(agent_loop, advisor_name).await;
        }

        Ok(Self {
            config,
            backend: backend_arc,
            agent_loop,
            validator,
            context,
        })
    }

    /// Build the named frontier advisor and attach it to the agent loop, or
    /// warn and return the loop unchanged.
    ///
    /// The advisor is a remote/hosted model, so enabling it means low-confidence
    /// prompts are sent off-host — we warn explicitly. Only `claude` is wired
    /// today (the article's advisor was Claude Opus); `openrouter` is a trivial
    /// follow-up once it grows an env constructor.
    async fn maybe_attach_advisor(agent_loop: AgentLoop, name: &str) -> AgentLoop {
        #[cfg(feature = "remote-backends")]
        match Self::create_advisor(name).await {
            Some(advisor) => {
                eprintln!(
                    "⚠  Frontier advisor '{}' enabled — low-confidence prompts will be sent \
                     off-host to a remote model.",
                    name
                );
                agent_loop.with_advisor(advisor)
            }
            None => agent_loop,
        }
        #[cfg(not(feature = "remote-backends"))]
        {
            let _ = name;
            eprintln!("⚠  --advisor requires the 'remote-backends' feature; ignoring.");
            agent_loop
        }
    }

    /// Resolve an advisor backend by name from the environment (API keys).
    /// Returns `None` (with a warning) when the backend can't be built.
    #[cfg(feature = "remote-backends")]
    async fn create_advisor(name: &str) -> Option<Arc<dyn CommandGenerator>> {
        match name.to_ascii_lowercase().as_str() {
            "claude" | "anthropic" => match crate::backends::remote::ClaudeBackend::from_env() {
                Ok(backend) => Some(Arc::new(backend)),
                Err(e) => {
                    eprintln!("⚠  advisor 'claude' unavailable: {}", e);
                    None
                }
            },
            other => {
                eprintln!(
                    "⚠  unknown advisor '{}': only 'claude' is supported today",
                    other
                );
                None
            }
        }
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

            // Create safety config from user's safety level preference
            let safety_config = SafetyConfig::from_level(user_config.safety_level);

            // Create embedded backend (used as fallback or primary)
            let embedded_backend = EmbeddedModelBackend::new()
                .map_err(|e| CliError::ConfigurationError {
                    message: format!("Failed to create embedded backend: {}", e),
                })?
                .with_safety_config(safety_config)
                .map_err(|e| CliError::ConfigurationError {
                    message: format!("Failed to apply safety config to embedded backend: {}", e),
                })?;

            let embedded_arc: Arc<EmbeddedModelBackend> = Arc::new(embedded_backend);

            // Resolve remote endpoint URLs from the `[backends]` config section,
            // falling back to the built-in localhost defaults when unset.
            #[cfg(feature = "remote-backends")]
            let backends_cfg = &user_config.backends;
            #[cfg(feature = "remote-backends")]
            let mesh_url_str = backends_cfg
                .mesh_url
                .as_deref()
                .unwrap_or("http://localhost:9337");
            #[cfg(feature = "remote-backends")]
            let ollama_url_str = backends_cfg
                .ollama_url
                .as_deref()
                .unwrap_or("http://localhost:11434");
            #[cfg(feature = "remote-backends")]
            let exo_url_str = backends_cfg
                .exo_url
                .as_deref()
                .unwrap_or("http://localhost:52415");
            #[cfg(feature = "remote-backends")]
            let vllm_url_str = backends_cfg
                .vllm_url
                .as_deref()
                .unwrap_or("http://localhost:8000");
            #[cfg(feature = "remote-backends")]
            let ai_horde_url_str = backends_cfg
                .ai_horde_url
                .as_deref()
                .unwrap_or(crate::backends::remote::ai_horde::AI_HORDE_DEFAULT_URL);
            #[cfg(feature = "remote-backends")]
            let ai_horde_key_str = backends_cfg
                .ai_horde_key
                .as_deref()
                .unwrap_or(crate::backends::remote::ai_horde::AI_HORDE_ANON_KEY);

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
                    "mesh" => {
                        use crate::backends::remote::MeshBackend;
                        use reqwest::Url;

                        let mesh_model = user_config
                            .model_name
                            .clone()
                            .unwrap_or_else(|| "mesh".to_string());
                        if let Ok(mesh_url) = Url::parse(mesh_url_str) {
                            let mesh_backend = MeshBackend::new(mesh_url, mesh_model)
                                .map_err(|e| CliError::ConfigurationError {
                                    message: format!("Failed to create Mesh-LLM backend: {}", e),
                                })?
                                .with_embedded_fallback(embedded_arc.clone());
                            if mesh_backend.is_available().await {
                                tracing::info!("Using Mesh-LLM backend (user preference)");
                                return Ok(Box::new(mesh_backend));
                            } else {
                                tracing::warn!(
                                    "Mesh-LLM backend not available, falling back to embedded"
                                );
                            }
                        }
                    }
                    #[cfg(feature = "remote-backends")]
                    "ai-horde" | "aihorde" | "horde" => {
                        use crate::backends::remote::AiHordeBackend;

                        let horde = AiHordeBackend::new(ai_horde_url_str, ai_horde_key_str)
                            .map_err(|e| CliError::ConfigurationError {
                                message: format!("Failed to create AI-Horde backend: {}", e),
                            })?
                            .with_embedded_fallback(embedded_arc.clone());
                        // AI-Horde is a public volunteer cluster: never silently
                        // probe-and-skip. Honor the explicit request and rely on
                        // the embedded fallback if the Horde is unreachable.
                        if horde.is_available().await {
                            tracing::info!("Using AI-Horde backend (user preference)");
                        } else {
                            tracing::warn!(
                                "AI-Horde heartbeat failed; will attempt anyway with embedded fallback"
                            );
                        }
                        return Ok(Box::new(horde));
                    }
                    #[cfg(feature = "remote-backends")]
                    "hybrid" => {
                        use crate::backends::hybrid::{ContextSanitizer, HybridBackend};
                        use crate::backends::remote::{AiHordeBackend, MeshBackend};
                        use reqwest::Url;

                        // Pick the remote enhancer (default: mesh).
                        let remote_kind = backends_cfg.hybrid_remote.as_deref().unwrap_or("mesh");
                        let remote: Arc<dyn CommandGenerator> = match remote_kind {
                            "ai-horde" | "aihorde" | "horde" => Arc::new(
                                AiHordeBackend::new(ai_horde_url_str, ai_horde_key_str).map_err(
                                    |e| CliError::ConfigurationError {
                                        message: format!(
                                            "Failed to create AI-Horde remote for hybrid: {}",
                                            e
                                        ),
                                    },
                                )?,
                            ),
                            _ => {
                                let mesh_model = user_config
                                    .model_name
                                    .clone()
                                    .unwrap_or_else(|| "mesh".to_string());
                                let mesh_url = Url::parse(mesh_url_str).map_err(|e| {
                                    CliError::ConfigurationError {
                                        message: format!("Invalid mesh URL for hybrid: {}", e),
                                    }
                                })?;
                                Arc::new(MeshBackend::new(mesh_url, mesh_model).map_err(|e| {
                                    CliError::ConfigurationError {
                                        message: format!(
                                            "Failed to create Mesh remote for hybrid: {}",
                                            e
                                        ),
                                    }
                                })?)
                            }
                        };

                        // Seed the sanitizer with the current identity so the
                        // username/hostname are redacted alongside paths/IPs.
                        let user = std::env::var("USER")
                            .or_else(|_| std::env::var("LOGNAME"))
                            .ok();
                        let host = std::env::var("HOSTNAME").ok();
                        let sanitizer =
                            ContextSanitizer::new().with_identity(user.as_deref(), host.as_deref());

                        let local: Arc<dyn CommandGenerator> = embedded_arc.clone();
                        let hybrid =
                            HybridBackend::new(local, remote, sanitizer, backends_cfg.allow_public);
                        tracing::info!(
                            "Using Hybrid backend (local sanitizer + {} enhancer, sanitize={})",
                            remote_kind,
                            !backends_cfg.allow_public
                        );
                        return Ok(Box::new(hybrid));
                    }
                    #[cfg(feature = "remote-backends")]
                    "ollama" => {
                        use crate::backends::remote::OllamaBackend;
                        use reqwest::Url;

                        if let Ok(ollama_url) = Url::parse(ollama_url_str) {
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

                        if let Ok(exo_url) = Url::parse(exo_url_str) {
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

                        if let Ok(vllm_url) = Url::parse(vllm_url_str) {
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
                    "mesh" | "ollama" | "exo" | "vllm" | "ai-horde" | "hybrid" => {
                        return Err(Self::remote_backend_unavailable_error(model));
                    }
                    _ => {
                        tracing::warn!("Unknown backend '{}', using auto-detect", model);
                    }
                }
            }

            // Auto-detect: try remote backends with embedded fallback
            #[cfg(feature = "remote-backends")]
            {
                use crate::backends::remote::{
                    ExoBackend, MeshBackend, OllamaBackend, VllmBackend,
                };
                use reqwest::Url;

                // Priority: Mesh-LLM > Exo cluster > Ollama > vLLM > Embedded
                if let Ok(mesh_url) = Url::parse(mesh_url_str) {
                    let mesh_model = user_config
                        .model_name
                        .clone()
                        .unwrap_or_else(|| "mesh".to_string());
                    let mesh_backend = MeshBackend::new(mesh_url, mesh_model)
                        .map_err(|e| CliError::ConfigurationError {
                            message: format!("Failed to create Mesh-LLM backend: {}", e),
                        })?
                        .with_embedded_fallback(embedded_arc.clone());

                    if mesh_backend.is_available().await {
                        tracing::info!("Using Mesh-LLM backend (auto-detected)");
                        return Ok(Box::new(mesh_backend));
                    }
                }

                if let Ok(exo_url) = Url::parse(exo_url_str) {
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

                if let Ok(ollama_url) = Url::parse(ollama_url_str) {
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

                if let Ok(vllm_url) = Url::parse(vllm_url_str) {
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
        // The accepted roster and the `--backend-info` table are driven by the
        // same slice so the two user-facing surfaces cannot drift (#1115).
        use crate::backends::CLI_SERVABLE_BACKENDS;

        let normalized = backend.to_lowercase();
        if CLI_SERVABLE_BACKENDS
            .iter()
            .any(|(name, _)| *name == normalized)
        {
            return Ok(());
        }

        // Provide helpful error with suggestions
        let suggestion = CLI_SERVABLE_BACKENDS
            .iter()
            .map(|(name, _)| *name)
            .find(|v| v.starts_with(normalized.as_str()) || normalized.starts_with(v))
            .map(|v| format!(". Did you mean '{}'?", v))
            .unwrap_or_default();

        let available = CLI_SERVABLE_BACKENDS
            .iter()
            .map(|(name, note)| format!("  - {}: {}", name, note))
            .collect::<Vec<_>>()
            .join("\n");

        Err(CliError::InvalidArgument {
            message: format!(
                "Unknown backend '{}'{}\n\nAvailable backends:\n{}\n\n\
                 Set via: --backend <name>, CARO_BACKEND env var, or config file",
                backend, suggestion, available
            ),
        })
    }

    /// Build the error returned when the user requests a remote backend
    /// in a binary that was compiled without the `remote-backends` feature.
    ///
    /// Returning a hard `CliError::ConfigurationError` (instead of a silent
    /// `tracing::warn!` and fallback to the embedded backend) preserves the
    /// contract a user expects when they pass `--backend <name>`. Tracked by
    /// [#1081](https://github.com/wildcard/caro/issues/1081).
    #[cfg_attr(feature = "remote-backends", allow(dead_code))]
    fn remote_backend_unavailable_error(backend: &str) -> CliError {
        CliError::ConfigurationError {
            message: format!(
                "Backend '{}' requires the 'remote-backends' feature, \
                 which is not compiled into this build.\n\n\
                 The default `cargo install caro` binary ships without remote \
                 backends. To use {}, build from source with:\n  \
                 cargo install caro --features remote-backends --locked\n\n\
                 Alternatively, use the embedded backend (no setup required):\n  \
                 caro --backend embedded \"<your prompt>\"",
                backend, backend
            ),
        }
    }

    /// Get list of available backend names
    pub fn available_backends() -> Vec<&'static str> {
        // Derived from the same source of truth as `validate_backend_name`
        // and `--backend-info` so all three agree (#1115).
        crate::backends::CLI_SERVABLE_BACKENDS
            .iter()
            .map(|(name, _)| *name)
            .collect()
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

        // Resolve approval mode: `--approval` flag > CARO_APPROVAL env > config.
        let approval_mode = args
            .approval()
            .and_then(|s| ApprovalMode::from_str(&s).ok())
            .or_else(|| {
                std::env::var("CARO_APPROVAL")
                    .ok()
                    .and_then(|s| ApprovalMode::from_str(&s).ok())
            })
            .unwrap_or(self.config.approval_mode);

        // `auto` mode auto-confirms confirmable commands (hard blocks still apply),
        // equivalent to always passing `-y`.
        let auto_confirm = args.confirm() || approval_mode == ApprovalMode::Auto;

        // Compute the approval decision. `prompt`/`auto` use the static matrix
        // unchanged; `smart` blends in a bounded LLM judge (hard floor: a
        // Critical static match is never relaxed; uncertain → static fallback).
        let block_message = || {
            format!(
                "Command blocked due to {} risk: {}",
                validation.risk_level,
                validation.warnings.join(", ")
            )
        };
        let (requires_confirmation, blocked_reason, smart_note) = match approval_mode {
            ApprovalMode::Smart => {
                let ctx = RiskJudgeContext {
                    shell,
                    cwd: std::env::current_dir()
                        .ok()
                        .map(|p| p.display().to_string()),
                    static_risk: validation.risk_level,
                    matched_patterns: validation.matched_patterns.clone(),
                };
                let judgment = self.backend.classify_risk(&generated.command, &ctx).await;
                let decision = crate::safety::blend_smart_decision(
                    validation.risk_level,
                    judgment.as_ref(),
                    safety_level,
                    auto_confirm,
                );
                let blocked = decision.blocked.then(block_message);
                (decision.requires_confirmation, blocked, decision.note)
            }
            ApprovalMode::Prompt | ApprovalMode::Auto => {
                let requires_confirmation =
                    validation.risk_level.requires_confirmation(safety_level) && !auto_confirm;
                let blocked = validation
                    .risk_level
                    .is_blocked(safety_level)
                    .then(block_message);
                (requires_confirmation, blocked, None)
            }
        };

        // Surface any smart-mode re-route note to the user / JSON consumers.
        if let Some(note) = &smart_note {
            warnings_list.push(note.clone());
        }

        // Determine if command passes safety checks
        let can_execute = blocked_reason.is_none() && !requires_confirmation;

        // Build confirmation prompt (use the smart note when the judge re-routed).
        let confirmation_prompt = if requires_confirmation {
            match &smart_note {
                Some(note) => format!(
                    "Command '{}' requires confirmation ({}). Proceed? (y/N)",
                    generated.command, note
                ),
                None => format!(
                    "Command '{}' requires confirmation due to {} risk. Proceed? (y/N)",
                    generated.command, validation.risk_level
                ),
            }
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

        // Generate detailed explanation if explain mode is enabled
        let explain_mode = args.explain();
        let detailed_explanation = if explain_mode {
            use crate::prompts::ExplainerPromptBuilder;
            let explainer = ExplainerPromptBuilder::new(CapabilityProfile::detect().await);
            Some(explainer.create_explanation(&generated.command, &prompt))
        } else {
            None
        };

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
            explain_mode,
            detailed_explanation,
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
            approval_mode: ApprovalMode::default(),
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

    #[test]
    fn test_validate_backend_name_matches_servable_roster() {
        // Pins the acceptor to the single source of truth so the
        // `--backend-info` table (which iterates the same slice) can never
        // advertise a backend that `--backend <name>` rejects. This is the
        // regression guard for issue #1115.
        for (name, _note) in crate::backends::CLI_SERVABLE_BACKENDS {
            assert!(
                CliApp::validate_backend_name(name).is_ok(),
                "advertised backend '{}' must be accepted by --backend",
                name
            );
        }

        // Enum variants that exist in `BackendType` but are NOT CLI-wired are
        // intentionally rejected — advertising them was the #1115 bug. If a
        // future PR wires one of these, add it to CLI_SERVABLE_BACKENDS (which
        // updates every surface at once) rather than special-casing here.
        for unwired in ["claude", "static", "openrouter", "mlx"] {
            assert!(
                CliApp::validate_backend_name(unwired).is_err(),
                "'{}' is not CLI-wired yet and must not be silently accepted",
                unwired
            );
        }
    }

    #[test]
    fn test_remote_backend_unavailable_error_message() {
        // Verifies the loud-error contract for issue #1081 — when a user
        // requests a remote backend but the binary lacks the feature, the
        // error message must (a) name the backend, (b) point to the fix
        // (build with --features remote-backends), and (c) suggest the
        // no-setup-required embedded alternative.
        let err = CliApp::remote_backend_unavailable_error("ollama");
        let msg = err.to_string();
        assert!(msg.contains("ollama"), "names the requested backend");
        assert!(msg.contains("remote-backends"), "names the missing feature");
        assert!(
            msg.contains("cargo install caro --features remote-backends"),
            "tells the user exactly how to rebuild"
        );
        assert!(
            msg.contains("caro --backend embedded"),
            "suggests the embedded alternative"
        );
    }
}
