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

use caro::config::ConfigManager;
use caro::models::{ServerConfig, ShellType};
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
    let server_config = load_server_config()?;

    info!(
        host = %server_config.host,
        port = %server_config.port,
        auth = server_config.auth_token.is_some(),
        "Starting caro-server"
    );

    // Detect shell type
    let shell_type = detect_shell();

    // Build app state (agent_loop is None for now -- backends require
    // platform detection and model loading which we'll wire up incrementally)
    let state = Arc::new(AppState {
        agent_loop: None,
        shell_type,
        start_time: Instant::now(),
    });

    // Build router (with or without auth)
    let app = if let Some(ref token) = server_config.auth_token {
        info!("Bearer token authentication enabled");
        server::build_router_with_auth(state, token.clone())
    } else {
        warn!("No auth token configured -- all requests will be accepted");
        server::build_router(state)
    };

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", server_config.host, server_config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("caro-server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Load server configuration from config file + environment overrides.
fn load_server_config() -> anyhow::Result<ServerConfig> {
    // Try loading from config file
    let mut config = match ConfigManager::new() {
        Ok(cm) => match cm.load() {
            Ok(uc) => uc.server,
            Err(_) => ServerConfig::default(),
        },
        Err(_) => ServerConfig::default(),
    };

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

    Ok(config)
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
