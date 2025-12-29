# Data Model: Core Infrastructure Modules

**Feature**: 003-implement-core-infrastructure
**Date**: 2025-10-02

## Entity Definitions

### 1. CachedModel

**Purpose**: Represents a locally stored model with metadata for integrity validation

**Fields**:
- `model_id`: String - Hugging Face model identifier (e.g., "meta-llama/Llama-2-7b")
- `path`: PathBuf - Absolute path to cached model file
- `checksum`: String - SHA256 hash for integrity verification
- `size_bytes`: u64 - File size in bytes
- `downloaded_at`: DateTime<Utc> - Timestamp of download completion
- `last_accessed`: DateTime<Utc> - Last time model was loaded from cache
- `version`: Option<String> - Model version/revision if available

**Validation Rules**:
- `model_id` must not be empty
- `path` must exist and be readable
- `checksum` must be valid 64-character hex SHA256
- `size_bytes` must match actual file size
- `downloaded_at` must be <= current time

**State Transitions**:
- Downloading → Validating → Cached
- Cached → Corrupted (if checksum fails)
- Cached → Evicted (if cache size limit reached)

---

### 2. CacheManifest

**Purpose**: Tracks all cached models with metadata for management and cleanup

**Fields**:
- `version`: String - Manifest schema version (e.g., "1.0.0")
- `models`: HashMap<String, CachedModel> - Map of model_id to CachedModel
- `total_size_bytes`: u64 - Sum of all cached model sizes
- `max_cache_size_bytes`: u64 - User-configurable cache limit
- `last_updated`: DateTime<Utc> - Last manifest modification time

**Validation Rules**:
- `version` must match supported schema versions
- `total_size_bytes` must equal sum of model sizes
- `max_cache_size_bytes` must be > 0
- All paths in `models` must exist

**Operations**:
- `add_model(model: CachedModel)` - Add new cached model
- `remove_model(model_id: &str)` - Evict model from cache
- `get_model(model_id: &str)` - Retrieve cached model metadata
- `cleanup_lru()` - Evict least-recently-used models if over size limit
- `validate_integrity()` - Check all checksums match

---

### 3. UserConfiguration

**Purpose**: User preferences and settings persisted across caro invocations

**Fields**:
- `default_shell`: Option<ShellType> - Preferred shell (overrides auto-detection)
- `safety_level`: SafetyLevel - Default safety level (Strict, Moderate, Permissive)
- `default_model`: Option<String> - Preferred model identifier
- `log_level`: LogLevel - Logging verbosity (Debug, Info, Warn, Error)
- `cache_max_size_gb`: u64 - Maximum cache size in GB (default 10)
- `log_rotation_days`: u32 - Days to keep logs (default 7)

**Validation Rules**:
- `safety_level` must be valid enum variant
- `default_shell` if present must be valid ShellType
- `log_level` must be valid enum variant
- `cache_max_size_gb` must be >= 1 and <= 1000
- `log_rotation_days` must be >= 1 and <= 365

**Defaults** (when config file missing):
```rust
UserConfiguration {
    default_shell: None,  // Auto-detect
    safety_level: SafetyLevel::Moderate,
    default_model: None,  // Use backend default
    log_level: LogLevel::Info,
    cache_max_size_gb: 10,
    log_rotation_days: 7,
}
```

**TOML Representation**:
```toml
[general]
safety_level = "Moderate"
default_shell = "bash"
default_model = "meta-llama/Llama-2-7b"

[logging]
log_level = "Info"
log_rotation_days = 7

[cache]
max_size_gb = 10
```

---

### 4. ConfigSchema

**Purpose**: Defines valid configuration keys and validation logic

**Fields**:
- `known_sections`: Vec<String> - Valid config file sections
- `known_keys`: HashMap<String, KeyValidator> - Map of key to validation function
- `deprecated_keys`: HashMap<String, String> - Old key → new key migration map

**Validation Rules**:
- Unknown sections generate warnings (forward compatibility)
- Unknown keys generate warnings (forward compatibility)
- Deprecated keys trigger migration suggestion
- Invalid values generate errors with helpful messages

**Operations**:
- `validate(config: &toml::Value) -> Result<(), ConfigError>` - Validate entire config
- `migrate(old_config: &toml::Value) -> toml::Value` - Migrate deprecated keys

---

### 5. ExecutionContext

**Purpose**: Environment state when caro is invoked for context-aware generation

**Fields**:
- `current_dir`: PathBuf - Working directory at invocation time
- `shell_type`: ShellType - Detected or configured shell
- `platform`: Platform - OS platform (Linux, MacOS, Windows)
- `environment_vars`: HashMap<String, String> - Relevant env vars (filtered)
- `username`: String - Current user name
- `hostname`: String - Machine hostname
- `captured_at`: DateTime<Utc> - Timestamp of context capture

**Validation Rules**:
- `current_dir` must be absolute path
- `shell_type` must be valid enum variant
- `platform` must be valid enum variant
- `environment_vars` must not contain sensitive patterns
- `username` and `hostname` must not be empty

**Environment Variable Filtering**:
- Include: PATH, HOME, USER, SHELL, LANG, PWD
- Exclude: Patterns matching API_KEY, TOKEN, PASSWORD, SECRET, CREDENTIAL

**Operations**:
- `capture() -> Result<ExecutionContext>` - Capture current environment
- `to_model_context() -> String` - Serialize for LLM prompt context

---

### 6. LogEntry

**Purpose**: Structured log event with metadata for observability

**Fields**:
- `timestamp`: DateTime<Utc> - When event occurred
- `level`: LogLevel - Severity (Debug, Info, Warn, Error)
- `target`: String - Module/component that logged (e.g., "caro::cache")
- `message`: String - Human-readable log message
- `operation_id`: Option<String> - Correlation ID for multi-step operations
- `metadata`: HashMap<String, serde_json::Value> - Additional structured context
- `duration_ms`: Option<u64> - Operation duration if applicable

**Validation Rules**:
- `timestamp` must be <= current time
- `level` must be valid enum variant
- `target` must match module naming convention
- `message` must not contain sensitive data (enforced by redaction layer)

**JSON Representation**:
```json
{
  "timestamp": "2025-10-02T12:34:56.789Z",
  "level": "INFO",
  "target": "caro::cache",
  "message": "Model cached successfully",
  "operation_id": "abc123",
  "metadata": {
    "model_id": "meta-llama/Llama-2-7b",
    "size_bytes": 13476234240,
    "checksum": "a1b2c3..."
  },
  "duration_ms": 4523
}
```

**Redaction Patterns**:
- API keys: `[REDACTED:API_KEY]`
- Tokens: `[REDACTED:TOKEN]`
- Passwords: `[REDACTED:PASSWORD]`
- Regex patterns: `(?i)(api[_-]?key|token|password|secret|credential)`

---

## Enums

### ShellType
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Sh,  // POSIX fallback
}
```

### Platform
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
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
```

### SafetyLevel (already defined in models module)
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafetyLevel {
    Strict,
    Moderate,
    Permissive,
}
```

---

## Relationships

```
UserConfiguration
    ├─> default_shell: Option<ShellType>
    ├─> safety_level: SafetyLevel
    └─> log_level: LogLevel

ExecutionContext
    ├─> shell_type: ShellType (from config or auto-detect)
    ├─> platform: Platform
    └─> current_dir: PathBuf

CacheManifest
    └─> models: HashMap<String, CachedModel>

LogEntry
    ├─> level: LogLevel
    └─> metadata: JSON (may contain model_id, config values, etc.)
```

---

## Data Flow

### Configuration Loading Flow
```
1. User invokes caro
2. ConfigManager::load() checks for config file at XDG location
3. If exists: Parse TOML → Validate → Deserialize to UserConfiguration
4. If missing: Return UserConfiguration::default()
5. CLI args override config values
6. Final Configuration passed to application
```

### Cache Access Flow
```
1. Backend requests model by model_id
2. CacheManager::get_model(model_id)
3. Check CacheManifest for model_id
4. If cached: Validate checksum → Update last_accessed → Return path
5. If not cached: Download → Validate → Add to manifest → Return path
6. CacheManifest persisted to disk (JSON)
```

### Execution Context Capture Flow
```
1. CLI starts, ExecutionContext::capture() called
2. Capture current_dir (std::env::current_dir)
3. Detect shell_type (SHELL env var → fallback detection)
4. Detect platform (cfg! macros or std::env::consts::OS)
5. Filter environment_vars (include list - exclude patterns)
6. Return ExecutionContext
7. Pass to command generator as prompt context
```

### Logging Flow
```
1. Module calls tracing::info!("message", field1 = value)
2. Tracing subscriber captures event
3. Redaction layer scans message and fields
4. JSON formatter serializes to LogEntry
5. Async appender writes to log file (non-blocking)
6. Daily rotation creates new file, archives old
```

---

## File Formats

### Cache Manifest File
**Location**: `~/.cache/caro/manifest.json`

```json
{
  "version": "1.0.0",
  "models": {
    "meta-llama/Llama-2-7b": {
      "model_id": "meta-llama/Llama-2-7b",
      "path": "/home/user/.cache/caro/models/meta-llama_Llama-2-7b",
      "checksum": "a1b2c3d4...",
      "size_bytes": 13476234240,
      "downloaded_at": "2025-10-01T10:30:00Z",
      "last_accessed": "2025-10-02T08:15:00Z",
      "version": "main"
    }
  },
  "total_size_bytes": 13476234240,
  "max_cache_size_bytes": 10737418240,
  "last_updated": "2025-10-02T08:15:00Z"
}
```

### Configuration File
**Location**: `~/.config/caro/config.toml`

```toml
[general]
safety_level = "Moderate"
default_shell = "bash"
default_model = "meta-llama/Llama-2-7b"

[logging]
log_level = "Info"
log_rotation_days = 7

[cache]
max_size_gb = 10
```

### Log File
**Location**: `~/.local/share/caro/logs/caro.2025-10-02.log`

```json
{"timestamp":"2025-10-02T12:34:56.789Z","level":"INFO","target":"caro::cache","message":"Model cached successfully","operation_id":"abc123","metadata":{"model_id":"meta-llama/Llama-2-7b","size_bytes":13476234240},"duration_ms":4523}
{"timestamp":"2025-10-02T12:35:01.234Z","level":"WARN","target":"caro::config","message":"Unknown config key ignored","metadata":{"key":"experimental_feature","section":"general"}}
```

---

## Memory Estimates

| Entity | Typical Size | Max Count | Total Memory |
|--------|--------------|-----------|--------------|
| CachedModel | ~300 bytes | 20 models | ~6 KB |
| CacheManifest | ~6 KB | 1 | 6 KB |
| UserConfiguration | ~200 bytes | 1 | 200 bytes |
| ExecutionContext | ~2 KB | 1 per request | 2 KB |
| LogEntry (in-memory queue) | ~500 bytes | 1000 (ring buffer) | 500 KB |

**Total Infrastructure Memory**: < 1 MB (negligible overhead)

---

## Validation Summary

All entities use serde with validation:
- Parse-time validation for config files
- Runtime validation for captured context
- Integrity validation for cached files
- Schema validation for log entries

Error messages are user-friendly and actionable:
- "Invalid safety_level 'high'. Valid options: Strict, Moderate, Permissive"
- "Cache directory not writable: /home/user/.cache/caro (Permission denied)"
- "Model checksum mismatch for 'model-id'. Re-download recommended."
