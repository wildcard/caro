# Canonical Patterns for AI Code Generation

> **Core Philosophy**: When there's exactly one way to do something, AI can't get it wrong. This document defines the canonical patterns for caro development.

## The One-Way Principle

Like nanolang's design philosophy, caro enforces canonical patterns:

> "When LLMs see multiple equivalent forms, they guess wrong ~50% of the time."

Each pattern below is **THE** way to do it. Not a suggestion. The only way.

---

## Pattern 1: Backend Implementation

**CANONICAL: Implement CommandGenerator trait**

```rust
use async_trait::async_trait;
use anyhow::Result;
use crate::backends::{CommandGenerator, BackendInfo, CommandRequest, GeneratedCommand};

pub struct MyBackend {
    config: BackendConfig,
    client: reqwest::Client,
}

#[async_trait]
impl CommandGenerator for MyBackend {
    async fn generate_command(&self, request: &CommandRequest) -> Result<GeneratedCommand> {
        // 1. Check availability first
        if !self.is_available().await {
            anyhow::bail!("Backend not available");
        }

        // 2. Make request
        let response = self.client
            .post(&self.config.base_url)
            .json(&request)
            .send()
            .await
            .context("Sending request to backend")?;

        // 3. Parse response
        let text = response.text().await?;
        extract_command_json(&text)
    }

    async fn is_available(&self) -> bool {
        self.client
            .get(&format!("{}/health", self.config.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "my_backend".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec!["generation".to_string()],
        }
    }
}
```

**FORBIDDEN alternatives:**
- Direct function calls without trait
- Synchronous implementations
- Panicking on errors

---

## Pattern 2: Error Handling

**CANONICAL: Context-rich errors with anyhow**

```rust
use anyhow::{anyhow, bail, Context, Result};

fn process_command(input: &str) -> Result<Command> {
    // Validation with context
    let parsed = parse_input(input)
        .context("Parsing user input")?;

    // Early return for invalid state
    if parsed.is_empty() {
        bail!("Empty command not allowed");
    }

    // Complex error with formatting
    let validated = validate_command(&parsed)
        .map_err(|e| anyhow!(
            "Command validation failed for '{}': {}",
            input.chars().take(50).collect::<String>(),
            e
        ))?;

    Ok(validated)
}
```

**FORBIDDEN alternatives:**
- `.unwrap()` in production code
- `panic!()` for recoverable errors
- Silent error swallowing with `let _ =`
- Bare `?` without context on fallible operations

---

## Pattern 3: Configuration Loading

**CANONICAL: Layered config with sensible defaults**

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub backend: BackendConfig,
    pub safety: SafetyConfig,
    pub output: OutputConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: BackendConfig::default(),
            safety: SafetyConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let default = Self::default();

        // Layer 1: File config
        let file_config = Self::load_from_file()
            .unwrap_or_else(|_| default.clone());

        // Layer 2: Environment overrides
        let with_env = file_config.apply_env_overrides();

        // Layer 3: Validation
        with_env.validate()?;

        Ok(with_env)
    }

    fn load_from_file() -> Result<Self> {
        let path = config_path()?;
        let content = std::fs::read_to_string(&path)
            .context("Reading config file")?;
        toml::from_str(&content)
            .context("Parsing config TOML")
    }

    fn apply_env_overrides(mut self) -> Self {
        if let Ok(timeout) = std::env::var("CARO_TIMEOUT") {
            if let Ok(secs) = timeout.parse() {
                self.backend.timeout_secs = secs;
            }
        }
        self
    }

    fn validate(&self) -> Result<()> {
        if self.backend.timeout_secs == 0 {
            bail!("Timeout must be greater than 0");
        }
        Ok(())
    }
}

fn config_path() -> Result<PathBuf> {
    dirs::config_dir()
        .ok_or_else(|| anyhow!("No config directory found"))?
        .join("caro")
        .join("config.toml")
        .pipe(Ok)
}
```

**FORBIDDEN alternatives:**
- Hardcoded configuration values
- Environment-only configuration
- Config without defaults
- Validation-free loading

---

## Pattern 4: Safety Validation

**CANONICAL: Pre-compiled patterns with RiskLevel**

```rust
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

lazy_static! {
    static ref CRITICAL_PATTERNS: Vec<(Regex, &'static str)> = vec![
        (Regex::new(r"rm\s+-rf\s+/\s*$").unwrap(), "Recursive delete of root"),
        (Regex::new(r"rm\s+-rf\s+~").unwrap(), "Recursive delete of home"),
        (Regex::new(r"mkfs\.").unwrap(), "Filesystem format"),
        (Regex::new(r":\(\)\{.*:\|:.*&.*\}.*;:").unwrap(), "Fork bomb"),
    ];

    static ref HIGH_PATTERNS: Vec<(Regex, &'static str)> = vec![
        (Regex::new(r"chmod\s+777").unwrap(), "World-writable permissions"),
        (Regex::new(r">\s*/dev/sd[a-z]").unwrap(), "Write to raw device"),
    ];
}

pub struct SafetyValidator;

impl SafetyValidator {
    pub fn validate(&self, cmd: &str) -> SafetyResult {
        // Check critical patterns first
        for (pattern, reason) in CRITICAL_PATTERNS.iter() {
            if pattern.is_match(cmd) {
                return SafetyResult {
                    allowed: false,
                    risk_level: RiskLevel::Critical,
                    reason: reason.to_string(),
                    matched_pattern: Some(pattern.to_string()),
                };
            }
        }

        // Then high patterns
        for (pattern, reason) in HIGH_PATTERNS.iter() {
            if pattern.is_match(cmd) {
                return SafetyResult {
                    allowed: true, // Allowed but needs confirmation
                    risk_level: RiskLevel::High,
                    reason: reason.to_string(),
                    matched_pattern: Some(pattern.to_string()),
                };
            }
        }

        // Default: safe
        SafetyResult {
            allowed: true,
            risk_level: RiskLevel::Safe,
            reason: String::new(),
            matched_pattern: None,
        }
    }
}
```

**FORBIDDEN alternatives:**
- Runtime pattern compilation
- String matching instead of regex
- Boolean-only safety checks
- Validation without reason tracking

---

## Pattern 5: Async Operations

**CANONICAL: Tokio with timeout and cancellation**

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout<T, F, Fut>(
    operation: F,
    timeout_duration: Duration,
    operation_name: &str,
) -> Result<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    timeout(timeout_duration, operation())
        .await
        .map_err(|_| anyhow!(
            "{} timed out after {}s",
            operation_name,
            timeout_duration.as_secs()
        ))?
}

// Usage
let result = fetch_with_timeout(
    || backend.generate_command(&request),
    Duration::from_secs(30),
    "Command generation",
).await?;
```

**FORBIDDEN alternatives:**
- Blocking operations in async context
- `std::thread::sleep` in async
- Infinite loops without timeout
- Ignoring cancellation

---

## Pattern 6: Test Structure

**CANONICAL: Arrange-Act-Assert with descriptive names**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_blocks_critical_rm_rf_root() {
        // Arrange
        let validator = SafetyValidator::new();
        let dangerous_cmd = "rm -rf /";

        // Act
        let result = validator.validate(dangerous_cmd);

        // Assert
        assert_eq!(result.risk_level, RiskLevel::Critical);
        assert!(!result.allowed);
        assert!(result.reason.contains("root"));
    }

    #[tokio::test]
    async fn backend_returns_valid_json_for_simple_prompt() {
        // Arrange
        let backend = TestBackend::new();
        let request = CommandRequest {
            prompt: "list files".to_string(),
            context: ExecutionContext::default(),
        };

        // Act
        let result = backend.generate_command(&request).await;

        // Assert
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(!cmd.cmd.is_empty());
    }
}
```

**Test naming convention:**
- `{unit}_returns_{expected}_for_{condition}`
- `{unit}_blocks_{what}_when_{condition}`
- `{unit}_fails_{how}_if_{condition}`

**FORBIDDEN alternatives:**
- Tests named `test_1`, `test_foo`
- Missing assertions
- Testing multiple behaviors in one test
- No error message in assertions

---

## Pattern 7: CLI Argument Parsing

**CANONICAL: Clap derive with documentation**

```rust
use clap::Parser;

/// Convert natural language to shell commands using AI
#[derive(Parser, Debug)]
#[command(name = "caro")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The natural language prompt to convert
    #[arg(required = true)]
    pub prompt: String,

    /// Target shell (bash, zsh, fish, sh, powershell, cmd)
    #[arg(short, long, default_value = "bash")]
    pub shell: String,

    /// Safety level (strict, moderate, permissive)
    #[arg(long, default_value = "moderate")]
    pub safety: SafetyLevel,

    /// Output format (json, yaml, plain)
    #[arg(short, long, default_value = "plain")]
    pub output: OutputFormat,

    /// Auto-confirm dangerous commands
    #[arg(short = 'y', long)]
    pub confirm: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
```

**FORBIDDEN alternatives:**
- Manual argument parsing
- Undocumented flags
- Inconsistent flag naming
- Missing defaults

---

## Pattern 8: JSON Response Parsing

**CANONICAL: Multi-strategy extraction**

```rust
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GeneratedCommand {
    pub cmd: String,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub explanation: String,
}

pub fn extract_command_json(response: &str) -> Result<GeneratedCommand> {
    // Strategy 1: Direct JSON parse
    if let Ok(cmd) = serde_json::from_str(response) {
        return Ok(cmd);
    }

    // Strategy 2: Extract from markdown code block
    let code_block_re = Regex::new(r"```(?:json)?\s*(\{[\s\S]*?\})\s*```").unwrap();
    if let Some(caps) = code_block_re.captures(response) {
        if let Ok(cmd) = serde_json::from_str(&caps[1]) {
            return Ok(cmd);
        }
    }

    // Strategy 3: Find bare JSON object
    let json_re = Regex::new(r#"\{[^{}]*"cmd"\s*:\s*"[^"]+(?:\\.[^"]*)*"[^{}]*\}"#).unwrap();
    if let Some(m) = json_re.find(response) {
        if let Ok(cmd) = serde_json::from_str(m.as_str()) {
            return Ok(cmd);
        }
    }

    // Strategy 4: Extract just the command string
    let cmd_re = Regex::new(r#""cmd"\s*:\s*"([^"]+(?:\\.[^"]*)*)""#).unwrap();
    if let Some(caps) = cmd_re.captures(response) {
        return Ok(GeneratedCommand {
            cmd: caps[1].to_string(),
            confidence: 0.5,
            explanation: String::new(),
        });
    }

    bail!("No valid command found in response: {}",
          response.chars().take(100).collect::<String>())
}
```

**FORBIDDEN alternatives:**
- Single-strategy parsing
- Ignoring malformed responses
- Panicking on parse failure

---

## Pattern 9: Logging

**CANONICAL: Structured logging with context**

```rust
use log::{debug, info, warn, error};

pub async fn process_request(request: &CommandRequest) -> Result<GeneratedCommand> {
    info!("Processing request: prompt_len={}", request.prompt.len());

    debug!("Full request: {:?}", request);

    let result = match backend.generate_command(request).await {
        Ok(cmd) => {
            info!("Generated command: risk={:?}", safety.risk_level(&cmd.cmd));
            debug!("Full command: {}", cmd.cmd);
            Ok(cmd)
        }
        Err(e) => {
            warn!("Backend failed: {}", e);
            error!("Request failed: prompt='{}', error={}",
                   request.prompt.chars().take(50).collect::<String>(),
                   e);
            Err(e)
        }
    };

    result
}
```

**Log levels:**
- `debug!` - Detailed internal state (development only)
- `info!` - Normal operations (production visible)
- `warn!` - Recoverable issues (needs attention)
- `error!` - Failures (requires investigation)

**FORBIDDEN alternatives:**
- `println!` for logging
- No log level discrimination
- Logging sensitive data (commands in production)

---

## Pattern 10: Platform Detection

**CANONICAL: Runtime detection with fallback**

```rust
#[derive(Debug, Clone)]
pub enum Os {
    MacOS,
    Linux,
    Windows,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Sh,
    PowerShell,
    Cmd,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub os: Os,
    pub arch: String,
    pub shell: Shell,
    pub cwd: PathBuf,
    pub available_commands: Vec<String>,
}

impl ExecutionContext {
    pub fn detect() -> Self {
        Self {
            os: Self::detect_os(),
            arch: std::env::consts::ARCH.to_string(),
            shell: Self::detect_shell(),
            cwd: std::env::current_dir().unwrap_or_default(),
            available_commands: Self::detect_commands(),
        }
    }

    fn detect_os() -> Os {
        match std::env::consts::OS {
            "macos" => Os::MacOS,
            "linux" => Os::Linux,
            "windows" => Os::Windows,
            _ => Os::Unknown,
        }
    }

    fn detect_shell() -> Shell {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| {
                if s.contains("zsh") { Some(Shell::Zsh) }
                else if s.contains("bash") { Some(Shell::Bash) }
                else if s.contains("fish") { Some(Shell::Fish) }
                else { None }
            })
            .unwrap_or(Shell::Unknown)
    }
}
```

**FORBIDDEN alternatives:**
- Compile-time only platform checks
- Hardcoded platform assumptions
- No fallback for unknown platforms

---

## Summary: The Canonical Way

| Operation | Canonical Pattern |
|-----------|------------------|
| Backend implementation | Implement `CommandGenerator` trait |
| Error handling | `anyhow` with `.context()` |
| Configuration | Layered with defaults |
| Safety validation | Pre-compiled regex patterns |
| Async operations | Tokio with timeout |
| Testing | Arrange-Act-Assert |
| CLI parsing | Clap derive |
| JSON parsing | Multi-strategy extraction |
| Logging | Structured with levels |
| Platform detection | Runtime with fallback |

**Follow these patterns and your code will work. Deviate and you invite bugs.**

---

*Last updated: 2026-01-12*
