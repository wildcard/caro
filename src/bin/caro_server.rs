//! caro-server: HTTP API server for cabinet integration.
//!
//! Exposes caro's command generation and safety validation pipeline
//! as a REST API for external consumers (cabinet agents, CI/CD, etc.).
//!
//! # Usage
//!
//! ```bash
//! # Build with server feature
//! cargo build --release --features server
//!
//! # Run with default config (~/.config/caro/config.toml)
//! caro-server
//!
//! # Override via environment variables
//! CARO_SERVER_HOST=0.0.0.0 CARO_SERVER_PORT=8080 caro-server
//!
//! # With auth token
//! CARO_SERVER_TOKEN=mysecret caro-server
//! ```

use caro::agent::AgentLoop;
use caro::backends::embedded::EmbeddedModelBackend;
use caro::backends::CommandGenerator;
use caro::config::ConfigManager;
use caro::context::ExecutionContext;
use caro::models::{ServerConfig, ShellType, UserConfiguration};
use caro::prompts::CapabilityProfile;
use caro::safety::SafetyConfig;
use caro::server::{self, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let user_config = load_user_config();
    let server_config = load_server_config(&user_config);

    info!(
        host = %server_config.host,
        port = %server_config.port,
        auth = server_config.auth_token.is_some(),
        "Starting caro-server"
    );

    // Detect shell type and execution context
    let shell_type = detect_shell();
    let context = ExecutionContext::detect();
    let profile = CapabilityProfile::detect_or_cached().await;

    // Initialize backend and agent loop
    let agent_loop = initialize_agent_loop(&user_config, context, profile).await;

    let state = Arc::new(AppState {
        agent_loop,
        shell_type,
        start_time: Instant::now(),
    });

    // Build router with auth + CORS based on config
    let app = server::build_router(state, &server_config);

    if server_config.auth_token.is_some() {
        info!("Bearer token authentication enabled");
    } else {
        warn!("No auth token configured -- all requests will be accepted");
    }
    if !server_config.allowed_origins.is_empty() {
        info!(origins = ?server_config.allowed_origins, "CORS enabled");
    }

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", server_config.host, server_config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("caro-server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize the AgentLoop with the best available backend.
async fn initialize_agent_loop(
    user_config: &UserConfiguration,
    context: ExecutionContext,
    profile: CapabilityProfile,
) -> Option<AgentLoop> {
    let safety_config = SafetyConfig::from_level(user_config.safety_level);

    // Try creating the embedded backend
    let backend: Arc<dyn CommandGenerator> = match EmbeddedModelBackend::new() {
        Ok(embedded) => {
            info!("Embedded backend initialized");
            Arc::new(embedded.with_safety_config(safety_config))
        }
        Err(e) => {
            warn!("Failed to initialize embedded backend: {}. Only static matcher will be available.", e);
            // Create a minimal mock that always fails -- static matcher will handle known queries
            return Some(
                AgentLoop::new(Arc::new(FallbackBackend), context, profile)
                    .with_static_matcher(true),
            );
        }
    };

    Some(
        AgentLoop::new(backend, context, profile).with_static_matcher(true),
    )
}

/// Minimal fallback backend when embedded model isn't available.
/// The static matcher handles known queries; this returns an error for everything else.
struct FallbackBackend;

#[async_trait::async_trait]
impl CommandGenerator for FallbackBackend {
    async fn generate_command(
        &self,
        _request: &caro::models::CommandRequest,
    ) -> Result<caro::models::GeneratedCommand, caro::backends::GeneratorError> {
        Err(caro::backends::GeneratorError::BackendUnavailable {
            reason: "No LLM backend available. Install a model or configure a remote backend."
                .to_string(),
        })
    }

    async fn is_available(&self) -> bool {
        false
    }

    fn backend_info(&self) -> caro::backends::BackendInfo {
        caro::backends::BackendInfo {
            backend_type: caro::models::BackendType::Mock,
            model_name: "fallback".to_string(),
            supports_streaming: false,
            max_tokens: 0,
            typical_latency_ms: 0,
            memory_usage_mb: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), caro::backends::GeneratorError> {
        Ok(())
    }
}

/// Load user configuration from config file.
fn load_user_config() -> UserConfiguration {
    match ConfigManager::new() {
        Ok(cm) => match cm.load() {
            Ok(uc) => uc,
            Err(e) => {
                warn!("Failed to load config: {}. Using defaults.", e);
                UserConfiguration::default()
            }
        },
        Err(e) => {
            warn!("Failed to init config manager: {}. Using defaults.", e);
            UserConfiguration::default()
        }
    }
}

/// Load server configuration with environment variable overrides.
fn load_server_config(user_config: &UserConfiguration) -> ServerConfig {
    let mut config = user_config.server.clone();

    // Environment variable overrides
    if let Ok(host) = std::env::var("CARO_SERVER_HOST") {
        config.host = host;
    }
    if let Ok(port) = std::env::var("CARO_SERVER_PORT") {
        if let Ok(p) = port.parse() {
            config.port = p;
        }
    }
    if let Ok(token) = std::env::var("CARO_SERVER_TOKEN") {
        config.auth_token = Some(token);
    }

    config
}

/// Detect the current shell type.
fn detect_shell() -> ShellType {
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            return ShellType::Zsh;
        }
        if shell.contains("fish") {
            return ShellType::Fish;
        }
        if shell.contains("bash") {
            return ShellType::Bash;
        }
    }
    ShellType::Bash
}
