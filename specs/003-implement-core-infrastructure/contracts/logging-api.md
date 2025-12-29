# Logging Module API Contract

**Module**: `caro::logging`
**Purpose**: Structured logging with tracing integration for observability

## Public API

### Logger

```rust
pub struct Logger {
    // Private fields
}

impl Logger {
    /// Initializes global tracing subscriber with configuration
    /// Should be called once at application startup
    /// Returns error if already initialized or config invalid
    pub fn init(config: LogConfig) -> Result<(), LogError>;

    /// Gets a logger instance for a specific module
    pub fn for_module(module: &str) -> Self;

    /// Creates operation span for tracking multi-step operations
    /// Returns guard that ends span on drop
    pub fn start_operation(&self, operation: &str) -> OperationSpan;
}
```

### LogConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub log_level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub rotation: LogRotation,
    pub redaction_enabled: bool,
}

impl LogConfig {
    /// Creates default configuration
    /// Level: Info, Format: Json, Output: File, Rotation: Daily, Redaction: Enabled
    pub fn default() -> Self;

    /// Builder pattern for custom configuration
    pub fn builder() -> LogConfigBuilder;

    /// Creates development-friendly configuration
    /// Level: Debug, Format: Pretty, Output: Stderr
    pub fn development() -> Self;

    /// Creates production configuration
    /// Level: Info, Format: Json, Output: File with daily rotation
    pub fn production() -> Self;
}
```

### LogConfigBuilder

```rust
pub struct LogConfigBuilder {
    // Private fields
}

impl LogConfigBuilder {
    pub fn new() -> Self;

    pub fn log_level(mut self, level: LogLevel) -> Self;

    pub fn format(mut self, format: LogFormat) -> Self;

    pub fn output(mut self, output: LogOutput) -> Self;

    pub fn rotation(mut self, rotation: LogRotation) -> Self;

    pub fn redaction(mut self, enabled: bool) -> Self;

    pub fn build(self) -> LogConfig;
}
```

### LogLevel

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Converts to tracing::Level
    pub fn to_tracing_level(&self) -> tracing::Level;

    /// Parses from string (case-insensitive)
    pub fn from_str(s: &str) -> Result<Self, LogError>;
}
```

### LogFormat

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogFormat {
    Json,   // Machine-readable structured logs
    Pretty, // Human-readable colored output
}
```

### LogOutput

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File { path: PathBuf },
}
```

### LogRotation

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogRotation {
    Never,
    Hourly,
    Daily,
    Weekly,
}
```

### OperationSpan

```rust
pub struct OperationSpan {
    // Private fields
}

impl OperationSpan {
    /// Records additional field in the span
    pub fn record(&self, key: &str, value: impl ToString);

    /// Records timing information
    pub fn record_duration(&self, duration: Duration);

    /// Records operation success
    pub fn success(&self);

    /// Records operation failure with error context
    pub fn error(&self, error: &dyn std::error::Error);
}

impl Drop for OperationSpan {
    /// Automatically ends span and logs duration
    fn drop(&mut self);
}
```

### Redaction

```rust
pub struct Redaction;

impl Redaction {
    /// Redacts sensitive patterns from a string
    pub fn redact(text: &str) -> String;

    /// Checks if text contains sensitive patterns
    pub fn contains_sensitive_data(text: &str) -> bool;

    /// Adds custom redaction pattern (regex)
    pub fn add_pattern(pattern: &str) -> Result<(), LogError>;
}
```

### LogError

```rust
#[derive(Error, Debug)]
pub enum LogError {
    #[error("Logger already initialized")]
    AlreadyInitialized,

    #[error("Log directory not writable: {0}")]
    DirectoryNotWritable(PathBuf),

    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),

    #[error("Invalid redaction pattern: {0}")]
    InvalidPattern(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Behavioral Contracts

### Contract: Logger Initialization

**Given**: Logger not yet initialized
**When**: `Logger::init(LogConfig::default())` is called
**Then**:
- Global tracing subscriber installed
- Returns `Ok(())`
- Subsequent logs are captured
- Cannot initialize again (returns `AlreadyInitialized`)

### Contract: Structured JSON Logging

**Given**: Logger initialized with `LogFormat::Json`
**When**: `tracing::info!("Test message", field1 = "value1")` is called
**Then**:
- Log output is valid JSON
- Contains: timestamp, level, target, message, field1
- Example:
```json
{"timestamp":"2025-10-02T12:34:56.789Z","level":"INFO","target":"caro::cache","message":"Test message","field1":"value1"}
```

### Contract: Pretty Console Logging

**Given**: Logger initialized with `LogFormat::Pretty`
**When**: `tracing::info!("Test message")` is called
**Then**:
- Log output is human-readable with colors
- Format: `2025-10-02 12:34:56 INFO caro::cache: Test message`
- Level colors: Debug=gray, Info=green, Warn=yellow, Error=red

### Contract: Log Level Filtering

**Given**: Logger initialized with `LogLevel::Warn`
**When**:
- `tracing::debug!("Debug")` called
- `tracing::info!("Info")` called
- `tracing::warn!("Warn")` called
**Then**:
- Debug and Info messages are NOT logged
- Warn message is logged

### Contract: File Output with Rotation

**Given**: Logger initialized with `LogOutput::File` and `LogRotation::Daily`
**When**: Logs are written over multiple days
**Then**:
- New file created each day: `caro.2025-10-02.log`, `caro.2025-10-03.log`
- Old logs retained up to `log_rotation_days` configuration
- Logs older than retention period automatically deleted

### Contract: Operation Span Tracking

**Given**: Logger initialized
**When**:
```rust
let span = logger.start_operation("cache_download");
// ... perform operation ...
span.record("model_id", "test-model");
span.record_duration(Duration::from_secs(4));
span.success();
drop(span);
```
**Then**:
- Span entry logged at start
- Span exit logged at drop with duration
- All recorded fields included in exit log
- JSON log includes `operation_id` for correlation

### Contract: Automatic Duration Logging

**Given**: OperationSpan created
**When**: Span is dropped after 2 seconds
**Then**:
- Log entry includes `duration_ms: 2000`
- Automatic timing without manual calculation

### Contract: Sensitive Data Redaction

**Given**: Logger with `redaction_enabled: true`
**When**: `tracing::info!("API key: {}", "sk_test_abc123")` is called
**Then**:
- Logged message: `"API key: [REDACTED:API_KEY]"`
- Original value not present in logs
- Pattern matching is case-insensitive

### Contract: Redaction Pattern Matching

**Given**: Redaction enabled
**When**: Various sensitive patterns logged
**Then**:
- `api_key=sk_test_123` → `api_key=[REDACTED:API_KEY]`
- `token: ghp_abc123` → `token: [REDACTED:TOKEN]`
- `password="secret"` → `password="[REDACTED:PASSWORD]"`
- `AWS_SECRET_ACCESS_KEY=xyz` → `AWS_SECRET_ACCESS_KEY=[REDACTED:SECRET]`

### Contract: Non-Blocking Async Logging

**Given**: Logger initialized with file output
**When**: Many log messages written quickly
**Then**:
- Main thread never blocks on I/O (NFR-004)
- Logs buffered in memory (ring buffer)
- Background writer thread flushes to disk
- Application performance unaffected

### Contract: Error Context Chains

**Given**: Nested error with context
**When**:
```rust
span.error(&error);
```
**Then**:
- Log includes full error chain
- Each error source logged separately
- Stack-like presentation of error context

### Contract: Log Directory Creation

**Given**: Log directory does not exist
**When**: Logger initialized with `LogOutput::File`
**Then**:
- Log directory created automatically
- Permissions set to 0700 (user-only)
- Returns `Ok(())`

### Contract: Invalid Configuration Rejection

**Given**: LogConfig with invalid log level string
**When**: `LogLevel::from_str("invalid")` is called
**Then**:
- Returns `Err(LogError::InvalidLogLevel("invalid"))`
- Error message lists valid levels: Debug, Info, Warn, Error

### Contract: Development vs Production Configs

**Given**: Different environment needs
**When**:
- `LogConfig::development()` used in dev
- `LogConfig::production()` used in prod
**Then**:
- Dev: Pretty format, stderr output, debug level
- Prod: JSON format, file output with rotation, info level

## Logging Macros Usage

The module exposes standard tracing macros:

```rust
// Basic logging
tracing::debug!("Debug message");
tracing::info!("Info message");
tracing::warn!("Warning message");
tracing::error!("Error message");

// With structured fields
tracing::info!(
    model_id = %model_id,
    size_bytes = file_size,
    "Model downloaded successfully"
);

// With spans
let _span = tracing::info_span!("cache_operation", operation = "download").entered();
tracing::info!("Starting download");
// ... work ...
tracing::info!("Download complete");
// Span auto-closed on drop

// Instrument functions
#[tracing::instrument]
async fn download_model(model_id: &str) -> Result<PathBuf> {
    // Function args automatically logged
    // Errors automatically logged on return
}
```

## Test Coverage Requirements

1. **Unit Tests**:
   - Log level filtering logic
   - Redaction pattern matching
   - Configuration validation
   - Log format serialization

2. **Integration Tests**:
   - End-to-end logging to file
   - Log rotation behavior
   - Span creation and timing
   - Async non-blocking writes

3. **Property Tests**:
   - Any string after redaction contains no sensitive patterns
   - JSON logs always deserialize successfully
   - Log level ordering (Debug < Info < Warn < Error)

## Performance Requirements

| Operation | Target Latency | Constraint |
|-----------|----------------|------------|
| `tracing::info!()` call | <1μs | Non-blocking, buffered |
| Span creation | <10μs | Stack allocation only |
| Redaction check | <100μs | Regex matching |
| Background flush | N/A | Async, doesn't block main thread |

## Thread Safety

- `Logger` is `Send + Sync` (global subscriber)
- Tracing subscriber is thread-safe by design
- Background writer uses async I/O with tokio
- No locks on hot logging path

## Error Handling

Logging errors are handled gracefully:
- I/O errors don't crash application (logged to stderr)
- Invalid configuration rejected at initialization
- Missing log directory created automatically
- Full disk handled with best-effort logging

## Redaction Patterns

Default sensitive patterns (case-insensitive regex):

```rust
static PATTERNS: &[&str] = &[
    r"(?i)(api[_-]?key)\s*[:=]\s*['\"]?([a-zA-Z0-9_-]+)",
    r"(?i)(token)\s*[:=]\s*['\"]?([a-zA-Z0-9_-]+)",
    r"(?i)(password)\s*[:=]\s*['\"]?([^'\"\\s]+)",
    r"(?i)(secret)\s*[:=]\s*['\"]?([a-zA-Z0-9_-]+)",
    r"(?i)(credential)\s*[:=]\s*['\"]?([a-zA-Z0-9_-]+)",
];
```

Replacement format: `$1=[REDACTED:${UPPERCASE($1)}]`
