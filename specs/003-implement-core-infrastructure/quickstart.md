# Quickstart: Core Infrastructure Modules

**Feature**: 003-implement-core-infrastructure
**Purpose**: Integration test scenarios validating cross-module functionality

## Test Scenarios

### Scenario 1: First-Time User Experience

**Objective**: Validate complete infrastructure initialization flow for new user

**Prerequisites**:
- No existing config file at `~/.config/caro/config.toml`
- No cache directory at `~/.cache/caro/`
- Clean test environment

**Steps**:
1. Initialize application with default configuration
2. Capture execution context
3. Attempt to load model from cache (should trigger download)
4. Generate command using captured context
5. Verify structured logs captured all operations

**Expected Results**:
```rust
// 1. Config loads with defaults
let config_manager = ConfigManager::new()?;
let config = config_manager.load()?;
assert_eq!(config.safety_level, SafetyLevel::Moderate);
assert_eq!(config.log_level, LogLevel::Info);

// 2. Context captures environment
let context = ExecutionContext::capture()?;
assert!(context.current_dir().is_absolute());
assert_ne!(context.shell_type(), ShellType::Sh); // Should detect actual shell
assert_ne!(context.username(), "");

// 3. Cache downloads model on first access
let cache_manager = CacheManager::new()?;
let model_path = cache_manager.get_model("test-model").await?;
assert!(model_path.exists());
assert!(cache_manager.is_cached("test-model"));

// 4. Logs contain all operations
// Verify JSON logs at ~/.local/share/caro/logs/caro.YYYY-MM-DD.log
// Should include: config_load, context_capture, model_download events
```

**Performance**:
- Total flow: <10 seconds (including model download)
- Config load: <100ms
- Context capture: <50ms
- Logging overhead: <1ms per event

---

### Scenario 2: Returning User with Cached Model

**Objective**: Validate offline operation and config persistence

**Prerequisites**:
- Existing config file with custom safety level: Strict
- Model already cached from previous run
- Network disconnected (offline mode)

**Steps**:
1. Load user configuration
2. Capture execution context
3. Load model from cache (no network request)
4. Verify config preferences applied
5. Check logs for offline operation

**Expected Results**:
```rust
// 1. Config loads custom values
let config = config_manager.load()?;
assert_eq!(config.safety_level, SafetyLevel::Strict);

// 2. Context captured successfully
let context = ExecutionContext::capture()?;
assert!(context.captured_at() <= Utc::now());

// 3. Cache serves model offline
let model_path = cache_manager.get_model("test-model").await?;
assert!(model_path.exists());
// Verify no network request made (check logs or mock)

// 4. Preferences applied
let final_safety = config.safety_level; // Should be Strict from config
assert_eq!(final_safety, SafetyLevel::Strict);

// 5. Logs show cache hit
// JSON log: {"level":"INFO","target":"caro::cache","message":"Model loaded from cache","duration_ms":45}
```

**Performance**:
- Total flow: <200ms (no download)
- Cache lookup: <100ms
- No network latency

---

### Scenario 3: CLI Argument Override

**Objective**: Validate that CLI args override config file settings

**Prerequisites**:
- Config file with `safety_level = "Moderate"`
- CLI args specify `--safety strict`

**Steps**:
1. Load configuration from file
2. Parse CLI arguments
3. Merge CLI args with config (CLI takes precedence)
4. Verify final configuration

**Expected Results**:
```rust
// 1. Load base config
let base_config = config_manager.load()?;
assert_eq!(base_config.safety_level, SafetyLevel::Moderate);

// 2. CLI args parsed
let cli_args = CliArgs {
    safety: Some("strict".to_string()),
    ..Default::default()
};

// 3. Merge with CLI precedence
let final_config = config_manager.merge_with_cli_args(base_config, &cli_args);

// 4. CLI arg wins
assert_eq!(final_config.safety_level, SafetyLevel::Strict);
```

---

### Scenario 4: Context-Aware Command Generation

**Objective**: Validate execution context is properly passed to command generator

**Prerequisites**:
- User in `/home/user/Downloads` directory
- Shell is `bash`
- Platform is Linux

**Steps**:
1. Capture execution context
2. Serialize context for LLM prompt
3. Verify context includes relevant details
4. Generate command using context

**Expected Results**:
```rust
// 1. Capture context
let context = ExecutionContext::capture()?;
assert_eq!(context.current_dir(), Path::new("/home/user/Downloads"));
assert_eq!(context.shell_type(), ShellType::Bash);
assert_eq!(context.platform(), Platform::Linux);

// 2. Serialize for prompt
let prompt_context = context.to_prompt_context();
assert!(prompt_context.contains("Downloads"));
assert!(prompt_context.contains("bash"));
assert!(prompt_context.contains("Linux"));

// 3. Context passed to generator
// CommandGenerator should receive context and generate:
// "find . -name '*.pdf'" (relative to Downloads, bash-compatible)
```

---

### Scenario 5: Structured Logging with Operations

**Objective**: Validate operation tracking and timing across modules

**Prerequisites**:
- Logger initialized with JSON format
- File output to test log directory

**Steps**:
1. Initialize logger with test configuration
2. Start cache operation span
3. Perform cache operation
4. End span and verify logs

**Expected Results**:
```rust
// 1. Init logger
let log_config = LogConfig::builder()
    .log_level(LogLevel::Debug)
    .format(LogFormat::Json)
    .output(LogOutput::File { path: test_log_path })
    .build();
Logger::init(log_config)?;

// 2. Create operation span
let logger = Logger::for_module("caro::cache");
let span = logger.start_operation("model_download");

// 3. Perform operation
cache_manager.get_model("test-model").await?;
span.record("model_id", "test-model");
span.record_duration(Duration::from_millis(4523));
span.success();

// 4. Verify JSON log
// Parse log file and assert:
// - Entry has operation_id
// - Duration logged: ~4523ms
// - All spans properly closed
// - No orphaned operations
```

---

### Scenario 6: Cache Size Limit and LRU Eviction

**Objective**: Validate cache size management and eviction

**Prerequisites**:
- Config with `cache_max_size_gb = 1`
- Multiple models totaling >1GB

**Steps**:
1. Download models until cache limit reached
2. Verify LRU eviction triggers
3. Check oldest model removed
4. Verify cache stats accurate

**Expected Results**:
```rust
// 1. Fill cache
cache_manager.get_model("model-a").await?; // 500MB
cache_manager.get_model("model-b").await?; // 500MB
cache_manager.get_model("model-c").await?; // 500MB (triggers eviction)

// 2. Verify eviction
let stats = cache_manager.stats();
assert!(stats.total_size_bytes <= 1_000_000_000); // <= 1GB

// 3. Oldest model removed
assert!(!cache_manager.is_cached("model-a")); // LRU victim
assert!(cache_manager.is_cached("model-b"));
assert!(cache_manager.is_cached("model-c"));

// 4. Stats accurate
assert_eq!(stats.total_models, 2);
```

---

### Scenario 7: Sensitive Data Redaction

**Objective**: Validate sensitive data never appears in logs

**Prerequisites**:
- Logger with redaction enabled
- Environment with API_KEY set

**Steps**:
1. Capture execution context (filters API_KEY)
2. Log message containing sensitive pattern
3. Verify logs contain redacted values only

**Expected Results**:
```rust
// 1. Context filters sensitive env vars
let context = ExecutionContext::capture()?;
assert!(!context.has_env_var("API_KEY"));
assert!(context.has_env_var("HOME")); // Non-sensitive included

// 2. Log with sensitive data
tracing::info!("Using API key: {}", "sk_test_abc123");

// 3. Verify redaction in log file
// JSON log entry:
// {"message":"Using API key: [REDACTED:API_KEY]",...}
// Original value "sk_test_abc123" NOT present
```

---

### Scenario 8: Configuration Validation and Error Handling

**Objective**: Validate config validation catches invalid values

**Prerequisites**:
- Config file with invalid safety_level value

**Steps**:
1. Attempt to load invalid config
2. Verify descriptive error message
3. Verify application doesn't crash

**Expected Results**:
```rust
// Invalid config.toml:
// [general]
// safety_level = "high"  # Invalid! Should be Strict/Moderate/Permissive

// 1. Load fails gracefully
let result = config_manager.load();
assert!(result.is_err());

// 2. Error message helpful
let error = result.unwrap_err();
assert!(error.to_string().contains("Invalid configuration value"));
assert!(error.to_string().contains("safety_level"));
assert!(error.to_string().contains("Strict, Moderate, Permissive"));

// 3. Application uses defaults instead
let fallback_config = UserConfiguration::default();
assert_eq!(fallback_config.safety_level, SafetyLevel::Moderate);
```

---

### Scenario 9: Cross-Module Integration Flow

**Objective**: Validate all modules work together in complete workflow

**Prerequisites**:
- Clean environment for full integration test

**Steps**:
1. Initialize all infrastructure modules
2. Load configuration
3. Capture execution context
4. Load model from cache
5. Generate command using context
6. Verify all operations logged

**Expected Results**:
```rust
#[tokio::test]
async fn test_full_infrastructure_integration() -> Result<()> {
    // 1. Initialize logging first
    Logger::init(LogConfig::development())?;

    // 2. Load config (defaults)
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    // 3. Capture context
    let context = ExecutionContext::capture()?;
    tracing::info!(
        current_dir = %context.current_dir().display(),
        shell = ?context.shell_type(),
        "Execution context captured"
    );

    // 4. Initialize cache
    let cache_manager = CacheManager::new()?;
    let model_path = cache_manager.get_model("test-model").await?;
    tracing::info!(
        model_path = %model_path.display(),
        "Model loaded from cache"
    );

    // 5. Simulate command generation (mock)
    let prompt_context = context.to_prompt_context();
    tracing::info!(
        context = %prompt_context,
        "Context prepared for command generation"
    );

    // 6. Verify all logs present
    // Check log file contains entries for:
    // - config_load
    // - context_capture
    // - cache_lookup / model_download
    // - command_generation

    Ok(())
}
```

**Performance**:
- Total integration test: <15 seconds
- All performance requirements met:
  - Config load: <100ms
  - Context capture: <50ms
  - Cache operations: <5s
  - Logging: non-blocking

---

## Running Quickstart Tests

### As Integration Tests

```bash
# Run all infrastructure integration tests
cargo test --test infrastructure_integration

# Run specific scenario
cargo test --test infrastructure_integration test_first_time_user_experience

# With verbose output
cargo test --test infrastructure_integration -- --nocapture
```

### As Manual Verification

```bash
# 1. Clean environment
rm -rf ~/.config/caro ~/.cache/caro ~/.local/share/caro

# 2. Run caro (triggers infrastructure initialization)
cargo run -- "list PDF files in Downloads"

# 3. Verify artifacts created
ls -la ~/.config/caro/config.toml
ls -la ~/.cache/caro/manifest.json
ls -la ~/.local/share/caro/logs/
```

---

## Success Criteria

All scenarios must:
1. ✅ Execute without panics or crashes
2. ✅ Meet performance requirements (documented above)
3. ✅ Produce valid JSON logs (parseable)
4. ✅ Handle errors gracefully with clear messages
5. ✅ Leave filesystem in consistent state
6. ✅ Pass on all target platforms (Linux, macOS, Windows)

---

## Test Data

### Sample Config File
```toml
[general]
safety_level = "Strict"
default_shell = "bash"
default_model = "meta-llama/Llama-2-7b"

[logging]
log_level = "Debug"
log_rotation_days = 3

[cache]
max_size_gb = 5
```

### Sample Log Output
```json
{"timestamp":"2025-10-02T12:34:56.789Z","level":"INFO","target":"caro::config","message":"Configuration loaded","metadata":{"source":"file","path":"/home/user/.config/caro/config.toml"},"duration_ms":15}
{"timestamp":"2025-10-02T12:34:56.804Z","level":"INFO","target":"caro::execution","message":"Execution context captured","metadata":{"current_dir":"/home/user/Downloads","shell":"bash","platform":"Linux"},"duration_ms":8}
{"timestamp":"2025-10-02T12:35:01.234Z","level":"INFO","target":"caro::cache","message":"Model cached successfully","operation_id":"abc123","metadata":{"model_id":"meta-llama/Llama-2-7b","size_bytes":13476234240},"duration_ms":4523}
```

---

**Test Suite Location**: `tests/integration/infrastructure_integration.rs`
**Estimated Total Test Time**: <60 seconds (including model downloads in CI cache)
