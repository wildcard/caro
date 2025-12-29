# Config Module API Contract

**Module**: `caro::config`
**Purpose**: Configuration management with CLI integration and user preferences

## Public API

### ConfigManager

```rust
pub struct ConfigManager {
    // Private fields
}

impl ConfigManager {
    /// Creates a new ConfigManager with default config path
    /// Returns error if config directory cannot be accessed
    pub fn new() -> Result<Self, ConfigError>;

    /// Creates a new ConfigManager with custom config path
    pub fn with_config_path(config_path: PathBuf) -> Result<Self, ConfigError>;

    /// Loads configuration from file or returns defaults if not found
    /// Returns error if config file exists but is invalid
    pub fn load(&self) -> Result<UserConfiguration, ConfigError>;

    /// Saves configuration to file
    pub fn save(&self, config: &UserConfiguration) -> Result<(), ConfigError>;

    /// Merges CLI arguments with loaded configuration
    /// CLI args take precedence over config file
    pub fn merge_with_cli_args(
        &self,
        config: UserConfiguration,
        cli_args: &CliArgs,
    ) -> UserConfiguration;

    /// Validates configuration without saving
    pub fn validate(&self, config: &UserConfiguration) -> Result<(), ConfigError>;

    /// Gets config file path
    pub fn config_path(&self) -> &Path;
}
```

### UserConfiguration

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserConfiguration {
    pub default_shell: Option<ShellType>,
    pub safety_level: SafetyLevel,
    pub default_model: Option<String>,
    pub log_level: LogLevel,
    pub cache_max_size_gb: u64,
    pub log_rotation_days: u32,
}

impl UserConfiguration {
    /// Creates configuration with sensible defaults
    pub fn default() -> Self;

    /// Builder pattern for programmatic construction
    pub fn builder() -> UserConfigurationBuilder;
}
```

### UserConfigurationBuilder

```rust
pub struct UserConfigurationBuilder {
    // Private fields
}

impl UserConfigurationBuilder {
    pub fn new() -> Self;

    pub fn default_shell(mut self, shell: ShellType) -> Self;

    pub fn safety_level(mut self, level: SafetyLevel) -> Self;

    pub fn default_model(mut self, model: impl Into<String>) -> Self;

    pub fn log_level(mut self, level: LogLevel) -> Self;

    pub fn cache_max_size_gb(mut self, size: u64) -> Self;

    pub fn log_rotation_days(mut self, days: u32) -> Self;

    pub fn build(self) -> Result<UserConfiguration, ConfigError>;
}
```

### ConfigError

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found at {0}")]
    FileNotFound(PathBuf),

    #[error("Config directory not accessible: {0}")]
    DirectoryNotAccessible(String),

    #[error("Invalid TOML syntax: {0}")]
    InvalidToml(#[from] toml::de::Error),

    #[error("Invalid configuration value for '{key}': {reason}")]
    InvalidValue { key: String, reason: String },

    #[error("Unknown config section: {0} (will be ignored)")]
    UnknownSection(String),

    #[error("Deprecated config key '{old}', use '{new}' instead")]
    DeprecatedKey { old: String, new: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Behavioral Contracts

### Contract: load Returns Defaults When Config Missing

**Given**: No config file exists at `~/.config/caro/config.toml`
**When**: `config_manager.load()` is called
**Then**:
- Returns `Ok(UserConfiguration::default())`
- No error is raised
- Default values: safety_level=Moderate, log_level=Info, cache_max_size_gb=10

### Contract: load Parses Valid Config File

**Given**: Valid TOML config file exists with `safety_level = "Strict"`
**When**: `config_manager.load()` is called
**Then**:
- Returns `Ok(UserConfiguration{safety_level: SafetyLevel::Strict, ...})`
- All fields parsed correctly
- Operation completes in <100ms

### Contract: load Fails on Invalid TOML Syntax

**Given**: Config file contains invalid TOML (e.g., unclosed quote)
**When**: `config_manager.load()` is called
**Then**:
- Returns `Err(ConfigError::InvalidToml(...))`
- Error message includes line number and syntax issue
- Suggests correcting syntax or deleting config file

### Contract: load Fails on Invalid Enum Value

**Given**: Config file has `safety_level = "high"` (invalid variant)
**When**: `config_manager.load()` is called
**Then**:
- Returns `Err(ConfigError::InvalidValue{key: "safety_level", reason: "..."})`
- Error message lists valid options: Strict, Moderate, Permissive
- File is not modified

### Contract: load Warns on Unknown Sections

**Given**: Config file has `[experimental]` section (not in schema)
**When**: `config_manager.load()` is called
**Then**:
- Returns `Ok(UserConfiguration)` (continues with known values)
- Warning logged: "Unknown config section: experimental (will be ignored)"
- Forward compatibility maintained

### Contract: save Persists Configuration

**Given**: UserConfiguration with custom values
**When**: `config_manager.save(&config)` is called
**Then**:
- Config file created at `~/.config/caro/config.toml`
- TOML serialization matches expected format
- File permissions set to 0600 (user read/write only)
- Returns `Ok(())`

### Contract: save Overwrites Existing Config

**Given**: Config file already exists with old values
**When**: `config_manager.save(&new_config)` is called
**Then**:
- Old file completely replaced with new values
- Atomic write (write to temp, rename)
- No data loss if write fails midway

### Contract: merge_with_cli_args Prioritizes CLI Args

**Given**: Config has `safety_level = Moderate`, CLI args have `--safety strict`
**When**: `config_manager.merge_with_cli_args(config, &cli_args)` is called
**Then**:
- Returned config has `safety_level = Strict`
- Other config values unchanged
- Original config not mutated

### Contract: merge_with_cli_args Uses Config Defaults

**Given**: Config has `default_shell = bash`, CLI args have no shell specified
**When**: `config_manager.merge_with_cli_args(config, &cli_args)` is called
**Then**:
- Returned config has `default_shell = Some(ShellType::Bash)`
- CLI args don't override if not provided

### Contract: validate Accepts Valid Configuration

**Given**: UserConfiguration with all valid values
**When**: `config_manager.validate(&config)` is called
**Then**:
- Returns `Ok(())`
- No modifications to config

### Contract: validate Rejects Out-of-Range Values

**Given**: UserConfiguration with `cache_max_size_gb = 0`
**When**: `config_manager.validate(&config)` is called
**Then**:
- Returns `Err(ConfigError::InvalidValue{key: "cache_max_size_gb", reason: "must be >= 1"})`

### Contract: Builder Pattern Construction

**Given**: UserConfigurationBuilder with method chain
**When**:
```rust
UserConfiguration::builder()
    .safety_level(SafetyLevel::Strict)
    .log_level(LogLevel::Debug)
    .build()
```
**Then**:
- Returns `Ok(UserConfiguration)` with specified values
- Unspecified fields use defaults

### Contract: Config Path Resolution

**Given**: Standard XDG environment
**When**: `ConfigManager::new()` is called
**Then**:
- Config path is `~/.config/caro/config.toml` (Linux/macOS)
- Config path is `%APPDATA%\caro\config.toml` (Windows)
- Parent directory created if missing

## TOML Format Specification

### Valid Configuration Example

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

### Field Mappings

| TOML Path | Rust Field | Type | Default |
|-----------|------------|------|---------|
| `general.safety_level` | `safety_level` | SafetyLevel | Moderate |
| `general.default_shell` | `default_shell` | Option<ShellType> | None |
| `general.default_model` | `default_model` | Option<String> | None |
| `logging.log_level` | `log_level` | LogLevel | Info |
| `logging.log_rotation_days` | `log_rotation_days` | u32 | 7 |
| `cache.max_size_gb` | `cache_max_size_gb` | u64 | 10 |

### Validation Rules

- `safety_level`: Must be "Strict", "Moderate", or "Permissive"
- `default_shell`: Must be "bash", "zsh", "fish", "powershell", "cmd", "sh"
- `log_level`: Must be "Debug", "Info", "Warn", "Error"
- `cache.max_size_gb`: Must be 1-1000
- `logging.log_rotation_days`: Must be 1-365

## Test Coverage Requirements

1. **Unit Tests**:
   - TOML parsing with valid/invalid inputs
   - Enum deserialization edge cases
   - Builder pattern validation
   - Default value initialization

2. **Integration Tests**:
   - Load → Modify → Save → Load roundtrip
   - CLI args merging with various combinations
   - XDG directory resolution on different platforms
   - Concurrent config reads (no writes)

3. **Property Tests**:
   - Any valid UserConfiguration serializes and deserializes identically
   - CLI merge is idempotent (merge twice = merge once)
   - Validation always accepts default configuration

## Performance Requirements

| Operation | Target Latency | Constraint |
|-----------|----------------|------------|
| `load()` | <100ms | File I/O + TOML parsing |
| `save()` | <50ms | TOML serialization + file write |
| `validate()` | <1ms | In-memory validation only |
| `merge_with_cli_args()` | <1ms | Struct field updates |

## Thread Safety

- `ConfigManager` is `Send + Sync`
- Concurrent reads are safe (no shared mutable state)
- Concurrent writes must be serialized by caller (no internal locking)

## Error Handling

All errors include:
- Specific field/key that caused the error
- Reason for rejection with valid options
- File path context when applicable
- Suggestions for fixing the issue
