# Rust CLI Best Practices

> Reference guide for building CLI applications in Rust, tailored for caro development.

## Clap Argument Parsing

### Derive Macro Pattern
```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "caro", version, about, long_about = None)]
pub struct CliArgs {
    /// The natural language prompt (can be unquoted)
    #[arg(trailing_var_arg = true, num_args = 0..)]
    pub prompt: Vec<String>,

    /// Target shell for command generation
    #[arg(short, long, env = "CARO_SHELL")]
    pub shell: Option<String>,

    /// Safety level: strict, moderate, permissive
    #[arg(long, default_value = "moderate")]
    pub safety: SafetyLevel,

    /// Output format
    #[arg(short, long, value_enum, default_value = "plain")]
    pub output: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}
```

### Value Enums
```rust
#[derive(Clone, Debug, ValueEnum)]
pub enum SafetyLevel {
    Strict,
    Moderate,
    Permissive,
}

impl Default for SafetyLevel {
    fn default() -> Self {
        Self::Moderate
    }
}
```

### Environment Variable Fallbacks
```rust
#[arg(long, env = "CARO_CONFIG")]
pub config: Option<PathBuf>,
```

## Error Handling

### Custom Error Types with thiserror
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Model not found at path: {path}")]
    ModelNotFound { path: PathBuf },

    #[error("Inference failed: {message}")]
    InferenceFailed { message: String },

    #[error("Connection failed to {url}: {source}")]
    ConnectionFailed {
        url: String,
        #[source]
        source: reqwest::Error,
    },
}
```

### Context with anyhow
```rust
use anyhow::{Context, Result};

fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;

    toml::from_str(&content)
        .context("Failed to parse config as TOML")
}
```

### Error Propagation Pattern
```rust
// Good: Propagate with context
pub async fn generate(&self, prompt: &str) -> Result<String, BackendError> {
    let response = self.client
        .post(&self.url)
        .json(&request)
        .send()
        .await
        .map_err(|e| BackendError::ConnectionFailed {
            url: self.url.clone(),
            source: e,
        })?;

    // ...
}

// Avoid: Losing error context
.map_err(|_| BackendError::InferenceFailed { message: "failed".into() })
```

## Async Patterns

### Async Trait
```rust
use async_trait::async_trait;

#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand, GeneratorError>;

    async fn is_available(&self) -> bool;

    fn backend_info(&self) -> BackendInfo;
}
```

### Timeout Handling
```rust
use tokio::time::{timeout, Duration};

let result = timeout(Duration::from_secs(30), backend.generate_command(&request))
    .await
    .map_err(|_| BackendError::Timeout)?
    .map_err(|e| BackendError::InferenceFailed { message: e.to_string() })?;
```

### Cancellation Safety
```rust
use tokio::select;

select! {
    result = backend.generate_command(&request) => {
        // Handle result
    }
    _ = tokio::signal::ctrl_c() => {
        eprintln!("Interrupted by user");
        std::process::exit(130);
    }
}
```

## User Interaction

### Colored Output
```rust
use colored::Colorize;

fn display_risk_level(level: RiskLevel) {
    match level {
        RiskLevel::Safe => println!("{}", "Safe".green()),
        RiskLevel::Moderate => println!("{}", "Moderate".yellow()),
        RiskLevel::High => println!("{}", "High".red().bold()),
        RiskLevel::Critical => println!("{}", "CRITICAL".red().bold().blink()),
    }
}
```

### Progress Indicators
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new_spinner();
pb.set_style(
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
);
pb.set_message("Loading model...");

// Do work...

pb.finish_with_message("Model loaded");
```

### User Confirmation
```rust
use dialoguer::Confirm;

if Confirm::new()
    .with_prompt("Execute this command?")
    .default(false)
    .interact()
    .unwrap_or(false)
{
    execute_command(&cmd)?;
}
```

## Configuration Management

### TOML with serde
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub backend: BackendConfig,

    #[serde(default)]
    pub safety: SafetyConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendConfig {
    #[serde(default = "default_primary")]
    pub primary: String,

    #[serde(default)]
    pub enable_fallback: bool,
}

fn default_primary() -> String {
    "embedded".into()
}
```

### XDG Directories
```rust
use directories::ProjectDirs;

fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("sh", "caro", "caro")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}
```

## Logging

### Tracing Setup
```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn setup_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::from_default_env()
            .add_directive("caro=info".parse().unwrap())
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();
}
```

### Structured Logging
```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(self), fields(backend = %self.name()))]
async fn generate_command(&self, request: &CommandRequest) -> Result<...> {
    debug!("Starting command generation");

    let result = self.inference(request).await;

    match &result {
        Ok(cmd) => info!(command = %cmd.command, "Generated command"),
        Err(e) => warn!(error = %e, "Generation failed"),
    }

    result
}
```

## Cross-Platform Considerations

### Path Handling
```rust
use std::path::{Path, PathBuf};

// Good: Use Path methods
let config_path = dirs::config_dir()
    .map(|p| p.join("caro").join("config.toml"));

// Avoid: String concatenation
let bad_path = format!("{}/.config/caro", std::env::var("HOME").unwrap());
```

### Platform Detection
```rust
#[cfg(target_os = "macos")]
fn default_shell() -> &'static str {
    "zsh"
}

#[cfg(target_os = "linux")]
fn default_shell() -> &'static str {
    "bash"
}

#[cfg(target_os = "windows")]
fn default_shell() -> &'static str {
    "powershell"
}
```

### Feature Flags for Platform-Specific Code
```rust
// Cargo.toml
[target.'cfg(all(target_os = "macos", target_arch = "aarch64"))'.dependencies]
mlx-rs = { version = "0.25", optional = true }

// Code
#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "embedded-mlx"))]
mod mlx_backend;
```

## Testing CLI Applications

### Integration Tests
```rust
use std::process::Command;

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("caro"));
}
```

### Argument Parsing Tests
```rust
use clap::Parser;

#[test]
fn test_unquoted_prompt() {
    let args = CliArgs::parse_from(["caro", "list", "all", "files"]);
    assert_eq!(args.prompt.join(" "), "list all files");
}

#[test]
fn test_with_flags() {
    let args = CliArgs::parse_from([
        "caro", "--shell", "zsh", "--verbose", "find", "large", "files"
    ]);
    assert_eq!(args.shell, Some("zsh".into()));
    assert!(args.verbose);
}
```

## Performance Tips

### Lazy Static Initialization
```rust
use once_cell::sync::Lazy;

static SAFETY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    DANGEROUS_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
});
```

### Avoid Unnecessary Allocations
```rust
// Good: Use references when possible
fn validate_command(cmd: &str) -> ValidationResult { }

// Good: Use Cow for optional ownership
use std::borrow::Cow;
fn normalize_command(cmd: &str) -> Cow<'_, str> {
    if cmd.contains('\t') {
        Cow::Owned(cmd.replace('\t', " "))
    } else {
        Cow::Borrowed(cmd)
    }
}
```

### Binary Size Optimization
```toml
# Cargo.toml [profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
strip = true        # Strip debug symbols
codegen-units = 1   # Single codegen unit
panic = "abort"     # Smaller binary
```
