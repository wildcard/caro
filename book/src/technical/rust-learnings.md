# Rust Learnings from SecretScout

**Source**: [globalbusinessadvisors/SecretScout](https://github.com/globalbusinessadvisors/SecretScout)

**Analysis Date**: 2025-11-09

**Project Overview**: SecretScout is a high-performance secret scanning tool written in Rust, designed as a complete rewrite of the gitleaks-action JavaScript project. It demonstrates production-grade Rust development with CLI, GitHub Actions, and WASM support.

## Key Performance Metrics

| Metric | JavaScript v2 | Rust v3 | Improvement |
|--------|---------------|---------|-------------|
| Cold start | ~25s | ~8s | **3x faster** |
| Warm start | ~12s | ~5s | **2.4x faster** |
| Memory usage | 512 MB | 200 MB | **60% less** |
| Binary size | N/A | ~4.6 MB | Optimized |

---

## 1. Project Architecture

### Clean Module Organization

```
secretscout/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # Binary entry point with mode detection
│   ├── error.rs            # Hierarchical error system
│   ├── binary/             # Gitleaks binary management
│   ├── cli/                # CLI argument parsing
│   ├── commands/           # detect & protect commands
│   ├── config/             # Configuration management
│   ├── events/             # GitHub event routing
│   ├── github/             # GitHub API client
│   ├── github_actions/     # Actions integration
│   ├── outputs/            # Summary & comments generation
│   ├── sarif/              # SARIF processing
│   └── wasm.rs             # WASM bindings
└── tests/
    └── integration_test.rs # End-to-end tests
```

**Key Principle**: **One module, one responsibility**

### Dual-Mode Binary Design

```rust
fn detect_mode() -> Mode {
    if env::var("GITHUB_ACTIONS").is_ok()
        && env::var("GITHUB_WORKSPACE").is_ok()
        && env::var("GITHUB_EVENT_PATH").is_ok()
    {
        Mode::GitHubActions
    } else {
        Mode::Cli
    }
}
```

**Application**: cmdai could adopt similar mode detection for:
- Interactive mode (TTY)
- Pipe mode (non-TTY)
- CI/CD mode (environment variables)

---

## 2. Error Handling Excellence

### Hierarchical Error System

```rust
pub enum Error {
    Config(ConfigError),
    Event(EventError),
    Binary(BinaryError),
    Sarif(SarifError),
    GitHub(GitHubError),
    Io(String),
    Json(String),
    Http(String),
}

impl Error {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Error::Config(ConfigError::MissingEnvVar(_)) => ErrorSeverity::Fatal,
            Error::Event(EventError::NoCommits) => ErrorSeverity::Expected,
            Error::Binary(BinaryError::CacheError(_)) => ErrorSeverity::NonFatal,
            _ => ErrorSeverity::Fatal,
        }
    }

    pub fn sanitized(&self) -> String {
        // Mask sensitive data in error messages
    }
}
```

**Key Learnings**:
1. **Nested error types** for each module domain
2. **Severity levels** (Fatal, NonFatal, Expected) for exit code mapping
3. **Sanitization methods** to prevent credential leaks
4. **thiserror** for ergonomic error derivation
5. **From trait** for automatic error conversion

**Application to cmdai**:
```rust
// Apply this pattern to cmdai errors
pub enum CmdaiError {
    Backend(BackendError),
    Safety(SafetyError),
    Config(ConfigError),
    Cache(CacheError),
}

impl CmdaiError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CmdaiError::Safety(SafetyError::CriticalRisk(_)) => ErrorSeverity::Fatal,
            CmdaiError::Backend(BackendError::Unavailable) => ErrorSeverity::NonFatal,
            _ => ErrorSeverity::Fatal,
        }
    }
}
```

---

## 3. Async/Await Patterns

### Retry with Exponential Backoff

```rust
async fn retry_with_backoff<F, Fut, T, E>(mut f: F) -> std::result::Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
{
    let max_retries = 3;
    let base_delay = Duration::from_secs(1);

    for attempt in 0..max_retries {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt < max_retries - 1 {
                    let delay = base_delay * 2_u32.pow(attempt);
                    tokio::time::sleep(delay).await;
                } else {
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}
```

**Application**: Use for HTTP backends (vLLM, Ollama) in cmdai
- Retry network failures with backoff
- Configurable max retries
- Generic implementation works with any async function

### Clean Async Signatures

```rust
// Clear, explicit async function signatures
pub async fn fetch_pr_commits(
    config: &Config,
    repository: &Repository,
    pr_number: i64,
) -> Result<Vec<Commit>>
```

**Best Practice**:
- Use `async fn` consistently
- Return `Result<T>` for fallible operations
- Pass configuration by reference (`&Config`)

---

## 4. Feature Flags for Cross-Platform Support

### Native vs WASM Feature Separation

```toml
[features]
default = ["native"]
native = [
    "tokio",
    "reqwest/native-tls",
    "octocrab",
    "flate2",
    "tar",
    "zip",
    "dirs",
]
wasm = [
    "wasm-bindgen",
    "wasm-bindgen-futures",
    "js-sys",
    "web-sys",
    "console_error_panic_hook",
    "serde-wasm-bindgen",
]

[dependencies]
# Core dependencies (always included)
serde = { workspace = true }
thiserror = { workspace = true }

# Optional platform-specific dependencies
tokio = { workspace = true, optional = true }
reqwest = { workspace = true, optional = true }
```

```rust
// Conditional compilation
#[cfg(feature = "native")]
pub mod commands;

#[cfg(feature = "wasm")]
pub mod wasm;
```

**Application to cmdai**:
```toml
[features]
default = ["mlx"]
mlx = ["cxx", "metal-rs"]  # Apple Silicon only
vllm = ["reqwest"]
ollama = ["reqwest"]
all-backends = ["mlx", "vllm", "ollama"]
```

---

## 5. Workspace-Level Dependency Management

### Centralized Version Control

```toml
# Root Cargo.toml
[workspace]
members = ["secretscout"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1.0", default-features = false, features = ["derive", "alloc"] }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
tokio = { version = "1.35", features = ["rt-multi-thread", "process", "fs", "io-util", "macros"] }
thiserror = "1.0"
```

```toml
# Member Cargo.toml
[dependencies]
serde = { workspace = true }
tokio = { workspace = true, optional = true }
```

**Benefits**:
- Single source of truth for versions
- Consistent feature flags across crates
- Easier dependency updates

**Application**: cmdai could adopt workspace structure if adding multiple crates (e.g., `cmdai-core`, `cmdai-mlx`, `cmdai-cli`)

---

## 6. Security Validation Patterns

### Input Validation

```rust
pub fn validate_git_ref(git_ref: &str) -> Result<()> {
    // Check for shell metacharacters
    let dangerous_chars = [';', '&', '|', '$', '`', '\n', '\r', '<', '>'];
    for ch in dangerous_chars {
        if git_ref.contains(ch) {
            return Err(ConfigError::InvalidGitRef(format!(
                "Contains dangerous character '{}'", ch
            )).into());
        }
    }

    // Check for path traversal
    if git_ref.contains("..") {
        return Err(ConfigError::InvalidGitRef(
            "Contains path traversal".to_string()
        ).into());
    }

    Ok(())
}
```

**Key Security Patterns**:
1. **Shell injection prevention**: Validate git refs, paths, arguments
2. **Path traversal prevention**: Check for `..` sequences
3. **HTML escaping**: Use proper escaping in outputs
4. **Sanitized error messages**: Remove sensitive data from logs

**Direct Application to cmdai**:
```rust
// Apply to generated commands in cmdai
pub fn validate_generated_command(cmd: &str) -> Result<()> {
    // Check for command injection patterns
    let dangerous_patterns = [
        ";", "&&", "||", "|",
        "$(", "`",
        "> /dev/", "< /dev/",
        "rm -rf /",
    ];

    for pattern in dangerous_patterns {
        if cmd.contains(pattern) {
            return Err(SafetyError::DangerousPattern(pattern.to_string()));
        }
    }

    Ok(())
}
```

---

## 7. Binary Optimization Strategies

### Release Profile Configuration

```toml
[profile.release]
opt-level = 'z'      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip debug symbols
panic = 'abort'      # Smaller panic handler
```

**Results**:
- Binary size: ~4.6 MB (optimized)
- Fast startup: ~8s cold start

**Application to cmdai**: Use similar profile for production builds targeting < 50MB binary size goal

---

## 8. Platform Detection Pattern

```rust
pub enum Platform {
    Linux,
    Darwin,
    Windows,
}

impl Platform {
    pub fn detect() -> Result<Self> {
        match std::env::consts::OS {
            "linux" => Ok(Platform::Linux),
            "macos" => Ok(Platform::Darwin),
            "windows" => Ok(Platform::Windows),
            other => Err(BinaryError::UnsupportedPlatform(other.to_string()).into()),
        }
    }

    pub fn archive_ext(&self) -> &'static str {
        match self {
            Platform::Windows => ".zip",
            _ => ".tar.gz",
        }
    }
}

pub enum Architecture {
    X64,
    Arm64,
}

impl Architecture {
    pub fn detect() -> Result<Self> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Architecture::X64),
            "aarch64" => Ok(Architecture::Arm64),
            other => Err(BinaryError::UnsupportedArchitecture(other.to_string()).into()),
        }
    }
}
```

**Application**: Use for MLX backend detection (Apple Silicon only)

---

## 9. Testing Best Practices

### Integration Test Structure

```rust
// Helper for test environment setup
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let event_path = temp_dir.path().join("event.json");
    (temp_dir, event_path)
}

// Realistic test fixtures
fn create_sarif_with_findings() -> serde_json::Value {
    serde_json::json!({
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "Gitleaks",
                        "version": "8.18.0"
                    }
                },
                "results": [
                    {
                        "ruleId": "aws-access-token",
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": "src/config.rs"
                                },
                                "region": {
                                    "startLine": 42
                                }
                            }
                        }],
                        "partialFingerprints": {
                            "commitSha": "abc123"
                        }
                    }
                ]
            }
        ]
    })
}

#[tokio::test]
async fn test_parse_sarif_report() {
    let (temp_dir, _) = setup_test_env();
    let sarif_json = create_sarif_with_findings();
    let report_path = temp_dir.path().join("results.sarif");

    fs::write(&report_path, serde_json::to_string_pretty(&sarif_json).unwrap()).unwrap();

    let findings = sarif::extract_findings(&report_path).await.unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "aws-access-token");
    assert_eq!(findings[0].file_path, "src/config.rs");
    assert_eq!(findings[0].line_number, 42);
}
```

**Testing Principles**:
1. **Realistic fixtures**: Use actual data structures from dependencies
2. **Temporary files**: Use `tempfile` crate for file-based tests
3. **Async tests**: Use `#[tokio::test]` for async code
4. **Helper functions**: Extract common setup into reusable functions
5. **Security tests**: Dedicated tests for injection prevention

**Application to cmdai**:
```rust
// Test safety validation with realistic prompts
#[tokio::test]
async fn test_detect_dangerous_rm_command() {
    let prompt = "delete everything in the root directory";
    let result = generate_command(prompt).await;

    assert!(result.is_err());
    match result {
        Err(SafetyError::CriticalRisk(msg)) => {
            assert!(msg.contains("rm -rf /"));
        }
        _ => panic!("Expected CriticalRisk error"),
    }
}
```

---

## 10. CLI Design with Clap

### Derive Macros for Modern CLI

```rust
#[derive(Parser, Debug)]
#[command(name = "secretscout")]
#[command(version, about = "Fast, memory-safe secret detection", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Detect {
        #[arg(short, long, default_value = ".")]
        source: PathBuf,

        #[arg(short, long, default_value = "results.sarif")]
        report_path: PathBuf,

        #[arg(short = 'f', long, default_value = "sarif")]
        report_format: String,

        #[arg(short, long)]
        config: Option<PathBuf>,

        #[arg(long)]
        baseline_path: Option<PathBuf>,

        #[arg(short, long)]
        verbose: bool,
    },
    Protect { /* ... */ },
    Version,
}
```

**Benefits**:
- Compile-time validation
- Automatic help generation
- Environment variable integration
- Type-safe argument parsing

**Application**: cmdai already uses clap, but could add:
- Subcommands for different modes
- Global verbose flag
- Config file argument

---

## 11. Caching Strategy

```rust
pub fn check_cache(version: &str, platform: Platform, arch: Architecture) -> Option<PathBuf> {
    let cache_dir = get_cache_dir().ok()?;
    let cache_key = get_cache_key(version, platform, arch);
    let cached_path = cache_dir.join(cache_key).join("gitleaks");

    if cached_path.exists() {
        Some(cached_path)
    } else {
        None
    }
}

pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(custom_dir) = env::var("SECRETSCOUT_CACHE_DIR") {
        PathBuf::from(custom_dir)
    } else {
        dirs::cache_dir()
            .ok_or_else(|| BinaryError::CacheError("Could not determine cache directory".into()))?
            .join("secretscout")
    };

    fs::create_dir_all(&cache_dir).map_err(|e| {
        BinaryError::CacheError(format!("Failed to create cache directory: {}", e))
    })?;

    Ok(cache_dir)
}

fn get_cache_key(version: &str, platform: Platform, arch: Architecture) -> String {
    format!("{}_{:?}_{:?}", version, platform, arch)
}
```

**Key Patterns**:
1. **Platform-specific cache keys**: Prevents conflicts
2. **Environment override**: `SECRETSCOUT_CACHE_DIR` for custom locations
3. **dirs crate**: Cross-platform cache directory detection
4. **Lazy creation**: Cache directory created on first use

**Direct Application to cmdai**:
```rust
// Apply to Hugging Face model caching
pub fn get_model_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(custom_dir) = env::var("CMDAI_CACHE_DIR") {
        PathBuf::from(custom_dir)
    } else {
        dirs::cache_dir()
            .ok_or_else(|| CacheError::InvalidDirectory)?
            .join("cmdai")
            .join("models")
    };

    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}
```

---

## 12. Type Safety with Domain Types

### Rich Domain Types

```rust
pub struct DetectedSecret {
    pub rule_id: String,
    pub file_path: String,
    pub line_number: u32,
    pub fingerprint: String,
    pub commit_sha: String,
    pub match_value: String,
}

impl DetectedSecret {
    pub fn generate_fingerprint(
        commit_sha: &str,
        file_path: &str,
        rule_id: &str,
        line_number: u32,
    ) -> String {
        format!("{}:{}:{}:{}", commit_sha, file_path, rule_id, line_number)
    }
}

// Serde integration for JSON parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifReport {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub version: String,
    #[serde(default)]
    pub runs: Vec<Run>,
}
```

**Best Practices**:
1. **Newtype pattern**: Create semantic types instead of using primitives
2. **Serde attributes**: Control serialization (`rename_all`, `default`, `rename`)
3. **Builder methods**: Associated functions for construction
4. **Type-safe conversions**: Implement `From`/`Into` traits

---

## 13. Dependency Injection Pattern

```rust
// Configuration passed by reference throughout
pub async fn run_detect(config: &Config) -> Result<()> {
    let event = events::parse_event_context(config).await?;
    let findings = scan_commits(config, &event).await?;
    generate_outputs(config, findings).await?;
    Ok(())
}

pub async fn scan_commits(config: &Config, event: &EventContext) -> Result<Vec<Finding>> {
    let binary_path = binary::ensure_binary_available(config).await?;
    // Use config throughout
}
```

**Benefits**:
- Testability: Easy to inject mock config
- Flexibility: Single source of configuration
- Performance: No cloning, just borrowing

---

## Key Takeaways for cmdai

### 1. Error Handling
- **Adopt hierarchical error types** with nested domain errors
- **Add severity levels** for better exit code mapping
- **Implement sanitization** for sensitive data in errors

### 2. Async Patterns
- **Add retry logic** for HTTP backends with exponential backoff
- **Use clean async signatures** throughout backend implementations

### 3. Feature Flags
- **Separate platform-specific code** with feature flags
- **Make MLX optional** for non-Apple Silicon builds

### 4. Security
- **Validate all inputs** for shell injection, path traversal
- **Add security tests** for dangerous command patterns
- **Sanitize outputs** to prevent credential leaks

### 5. Testing
- **Create realistic fixtures** for model responses
- **Use helper functions** for test environment setup
- **Test security validations** explicitly

### 6. Performance
- **Optimize release profile** for size and speed
- **Implement caching** for models with platform-specific keys
- **Use lazy loading** for expensive resources

### 7. Platform Support
- **Platform/architecture enums** for type-safe detection
- **Conditional compilation** for platform-specific code
- **Cross-platform directory handling** with `dirs` crate

### 8. Type Safety
- **Rich domain types** instead of primitives
- **Serde integration** for JSON parsing
- **From/Into traits** for conversions

---

## Code Patterns to Adopt Immediately

### 1. Error Severity Pattern
```rust
// Apply to cmdai/src/error.rs
impl CmdaiError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CmdaiError::Safety(SafetyError::CriticalRisk(_)) => ErrorSeverity::Fatal,
            CmdaiError::Backend(BackendError::Unavailable) => ErrorSeverity::NonFatal,
            CmdaiError::Config(ConfigError::MissingEnvVar(_)) => ErrorSeverity::Fatal,
            _ => ErrorSeverity::Fatal,
        }
    }
}
```

### 2. Retry with Backoff
```rust
// Apply to cmdai/src/backends/vllm.rs and ollama.rs
async fn retry_with_backoff<F, Fut, T>(mut f: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let base_delay = Duration::from_secs(1);

    for attempt in 0..max_retries {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt < max_retries - 1 {
                    let delay = base_delay * 2_u32.pow(attempt);
                    tokio::time::sleep(delay).await;
                } else {
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}
```

### 3. Platform Detection
```rust
// Apply to cmdai/src/backends/mlx.rs
pub enum Platform {
    AppleSilicon,
    IntelMac,
    Linux,
    Windows,
}

impl Platform {
    pub fn detect() -> Result<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => Ok(Platform::AppleSilicon),
            ("macos", "x86_64") => Ok(Platform::IntelMac),
            ("linux", _) => Ok(Platform::Linux),
            ("windows", _) => Ok(Platform::Windows),
            (os, arch) => Err(BackendError::UnsupportedPlatform(format!("{}/{}", os, arch))),
        }
    }

    pub fn supports_mlx(&self) -> bool {
        matches!(self, Platform::AppleSilicon)
    }
}
```

### 4. Input Validation
```rust
// Apply to cmdai/src/safety/mod.rs
pub fn validate_command_safety(cmd: &str) -> Result<SafetyLevel> {
    // Check for shell injection patterns
    let injection_patterns = [";", "&&", "||", "|", "$(", "`"];
    for pattern in injection_patterns {
        if cmd.contains(pattern) {
            return Err(SafetyError::ShellInjectionDetected(pattern.to_string()));
        }
    }

    // Check for dangerous operations
    let critical_patterns = ["rm -rf /", "mkfs", "dd if=/dev/zero", ":(){ :|:& };:"];
    for pattern in critical_patterns {
        if cmd.contains(pattern) {
            return Ok(SafetyLevel::Critical);
        }
    }

    Ok(SafetyLevel::Safe)
}
```

---

## References

- **Repository**: https://github.com/globalbusinessadvisors/SecretScout
- **Performance Benchmarks**: 3x faster cold start, 60% less memory vs JavaScript
- **Key Technologies**: Rust, Tokio, Octocrab, Serde, Clap, WASM
- **Architecture**: Dual-mode (CLI + GitHub Actions), multi-platform, WASM support
- **Size**: ~4.6 MB optimized binary
