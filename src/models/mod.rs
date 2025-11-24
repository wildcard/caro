// Models module - Core data structures
// These are placeholder stubs - tests should fail until proper implementation

use serde::{Deserialize, Serialize};

/// Request for command generation from natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Natural language description of desired command
    pub input: String,

    /// Target shell type for command generation
    pub shell: ShellType,

    /// User's safety preference level
    pub safety_level: SafetyLevel,

    /// Optional additional context (current directory, environment info)
    pub context: Option<String>,

    /// Optional backend preference
    pub backend_preference: Option<String>,
}

impl CommandRequest {
    /// Create a new command request with the given input and shell type
    pub fn new(input: impl Into<String>, shell: ShellType) -> Self {
        let input = input.into();
        let trimmed = input.trim().to_string();

        Self {
            input: trimmed,
            shell,
            safety_level: SafetyLevel::default(),
            context: None,
            backend_preference: None,
        }
    }

    /// Set the safety level (builder pattern)
    pub fn with_safety(mut self, level: SafetyLevel) -> Self {
        self.safety_level = level;
        self
    }

    /// Set the context (builder pattern)
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Set the backend preference (builder pattern)
    pub fn with_backend(mut self, backend: impl Into<String>) -> Self {
        self.backend_preference = Some(backend.into());
        self
    }

    /// Validate that the request is well-formed
    pub fn validate(&self) -> Result<(), String> {
        if self.input.is_empty() {
            return Err("Input cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Response from command generation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCommand {
    /// The generated shell command
    pub command: String,

    /// Human-readable explanation of what the command does
    pub explanation: String,

    /// Assessed risk level of the command
    pub safety_level: RiskLevel,

    /// Description of estimated impact
    pub estimated_impact: String,

    /// Alternative commands that could achieve similar results
    pub alternatives: Vec<String>,

    /// Name of the backend that generated this command
    pub backend_used: String,

    /// Time taken to generate the command in milliseconds
    pub generation_time_ms: u64,

    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f64,
}

impl GeneratedCommand {
    /// Validate that the generated command is well-formed
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_empty() {
            return Err("Command cannot be empty".to_string());
        }
        if self.explanation.is_empty() {
            return Err("Explanation cannot be empty".to_string());
        }
        if !(0.0..=1.0).contains(&self.confidence_score) {
            return Err(format!(
                "Confidence score must be between 0.0 and 1.0, got {}",
                self.confidence_score
            ));
        }
        Ok(())
    }
}

impl std::fmt::Display for GeneratedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;

        writeln!(f, "{}", "Generated Command:".bold())?;
        writeln!(f, "  {}", self.command.bright_cyan().bold())?;
        writeln!(f)?;
        writeln!(f, "{}", "Explanation:".bold())?;
        writeln!(f, "  {}", self.explanation)?;
        writeln!(f)?;
        writeln!(f, "{} {}", "Risk Level:".bold(), self.safety_level)?;
        writeln!(f, "{} {}", "Backend:".bold(), self.backend_used)?;
        writeln!(
            f,
            "{} {:.0}%",
            "Confidence:".bold(),
            self.confidence_score * 100.0
        )?;

        if !self.alternatives.is_empty() {
            writeln!(f)?;
            writeln!(f, "{}", "Alternatives:".bold())?;
            for alt in &self.alternatives {
                writeln!(f, "  • {}", alt.dimmed())?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

impl RiskLevel {
    /// Check if this risk level requires user confirmation at the given safety level
    pub fn requires_confirmation(&self, safety_level: SafetyLevel) -> bool {
        match safety_level {
            SafetyLevel::Strict => matches!(self, Self::Moderate | Self::High | Self::Critical),
            SafetyLevel::Moderate => matches!(self, Self::High | Self::Critical),
            SafetyLevel::Permissive => matches!(self, Self::Critical),
        }
    }

    /// Check if this risk level should be blocked at the given safety level
    pub fn is_blocked(&self, safety_level: SafetyLevel) -> bool {
        match safety_level {
            SafetyLevel::Strict => matches!(self, Self::High | Self::Critical),
            SafetyLevel::Moderate => matches!(self, Self::Critical),
            SafetyLevel::Permissive => false,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;
        match self {
            Self::Safe => write!(f, "{}", "Safe".green()),
            Self::Moderate => write!(f, "{}", "Moderate".yellow()),
            Self::High => write!(f, "{}", "High".bright_red()),
            Self::Critical => write!(f, "{}", "Critical".red().bold()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SafetyLevel {
    /// Blocks High and Critical commands, confirms Moderate
    Strict,
    /// Blocks Critical commands, confirms High
    #[default]
    Moderate,
    /// Warns about all dangerous commands but allows with confirmation
    Permissive,
}

impl std::str::FromStr for SafetyLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "strict" => Ok(SafetyLevel::Strict),
            "moderate" => Ok(SafetyLevel::Moderate),
            "permissive" => Ok(SafetyLevel::Permissive),
            _ => Err(format!(
                "Invalid safety level '{}'. Valid values: strict, moderate, permissive",
                s
            )),
        }
    }
}

impl std::fmt::Display for SafetyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Moderate => write!(f, "moderate"),
            Self::Permissive => write!(f, "permissive"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackendType {
    /// Mock backend for testing
    Mock,
    /// Embedded model backend (Qwen with MLX/CPU)
    Embedded,
    /// Ollama local LLM backend
    Ollama,
    /// vLLM HTTP API backend
    VLlm,
    /// Apple Silicon MLX backend (legacy)
    Mlx,
}

impl std::str::FromStr for BackendType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mock" => Ok(Self::Mock),
            "embedded" => Ok(Self::Embedded),
            "ollama" => Ok(Self::Ollama),
            "vllm" => Ok(Self::VLlm),
            "mlx" => Ok(Self::Mlx),
            _ => Err(format!("Unknown backend type: {}", s)),
        }
    }
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mock => write!(f, "mock"),
            Self::Embedded => write!(f, "embedded"),
            Self::Ollama => write!(f, "ollama"),
            Self::VLlm => write!(f, "vllm"),
            Self::Mlx => write!(f, "mlx"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Sh,
    PowerShell,
    Cmd,
    Unknown,
}

impl ShellType {
    /// Detect the current shell from environment
    pub fn detect() -> Self {
        // Check SHELL environment variable on Unix-like systems
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("bash") {
                return Self::Bash;
            } else if shell.contains("zsh") {
                return Self::Zsh;
            } else if shell.contains("fish") {
                return Self::Fish;
            } else if shell.ends_with("/sh") {
                return Self::Sh;
            }
        }

        // Check for Windows shells
        #[cfg(target_os = "windows")]
        {
            if std::env::var("PSModulePath").is_ok() {
                return Self::PowerShell;
            }
            return Self::Cmd;
        }

        Self::Unknown
    }

    /// Check if this is a POSIX-compatible shell
    pub fn is_posix(&self) -> bool {
        matches!(self, Self::Bash | Self::Zsh | Self::Fish | Self::Sh)
    }

    /// Check if this is a Windows shell
    pub fn is_windows(&self) -> bool {
        matches!(self, Self::PowerShell | Self::Cmd)
    }
}

impl Default for ShellType {
    fn default() -> Self {
        Self::detect()
    }
}

impl std::str::FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            "sh" => Ok(Self::Sh),
            "powershell" | "pwsh" => Ok(Self::PowerShell),
            "cmd" => Ok(Self::Cmd),
            _ => Ok(Self::Unknown),
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bash => write!(f, "bash"),
            Self::Zsh => write!(f, "zsh"),
            Self::Fish => write!(f, "fish"),
            Self::Sh => write!(f, "sh"),
            Self::PowerShell => write!(f, "powershell"),
            Self::Cmd => write!(f, "cmd"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Backend metadata for diagnostics and selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    /// Type of backend
    pub backend_type: BackendType,

    /// Name of the model being used
    pub model_name: String,

    /// Whether this backend supports streaming responses
    pub supports_streaming: bool,

    /// Maximum number of tokens the model can generate
    pub max_tokens: u32,

    /// Typical latency in milliseconds
    pub typical_latency_ms: u64,

    /// Memory usage in megabytes
    pub memory_usage_mb: u64,

    /// Backend version string
    pub version: String,
}

impl BackendInfo {
    /// Validate that backend info has reasonable values
    pub fn validate(&self) -> Result<(), String> {
        if self.model_name.is_empty() {
            return Err("Model name cannot be empty".to_string());
        }
        if self.max_tokens == 0 {
            return Err("Max tokens must be positive".to_string());
        }
        if self.version.is_empty() {
            return Err("Version cannot be empty".to_string());
        }
        Ok(())
    }
}

// All types are public through mod.rs exports

// ============================================================================
// Infrastructure Models (Feature 003)
// ============================================================================

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

/// Platform operating system type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

impl Platform {
    /// Detect current platform at runtime
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        compile_error!("Unsupported platform");
    }

    /// Check if platform is POSIX-compliant
    pub fn is_posix(&self) -> bool {
        matches!(self, Platform::Linux | Platform::MacOS)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Linux => write!(f, "Linux"),
            Platform::MacOS => write!(f, "macOS"),
            Platform::Windows => write!(f, "Windows"),
        }
    }
}

/// Log severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Convert to tracing Level
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" | "err" => Ok(LogLevel::Error),
            _ => Err(format!(
                "Invalid log level '{}'. Valid options: debug, info, warn, error",
                s
            )),
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Cached model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModel {
    pub model_id: String,
    pub path: PathBuf,
    pub checksum: String,
    pub size_bytes: u64,
    pub downloaded_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub version: Option<String>,
}

impl CachedModel {
    /// Validate cached model metadata
    pub fn validate(&self) -> Result<(), String> {
        if self.model_id.is_empty() {
            return Err("Model ID cannot be empty".to_string());
        }
        if self.checksum.len() != 64 {
            return Err(format!(
                "Checksum must be 64 characters (SHA256 hex), got {}",
                self.checksum.len()
            ));
        }
        if !self.checksum.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Checksum must be valid hexadecimal".to_string());
        }
        Ok(())
    }
}

/// Cache manifest tracking all cached models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManifest {
    pub version: String,
    pub models: HashMap<String, CachedModel>,
    pub total_size_bytes: u64,
    pub max_cache_size_bytes: u64,
    pub last_updated: DateTime<Utc>,
}

impl CacheManifest {
    /// Create a new empty manifest
    pub fn new(max_size_gb: u64) -> Self {
        Self {
            version: "1.0.0".to_string(),
            models: HashMap::new(),
            total_size_bytes: 0,
            max_cache_size_bytes: max_size_gb * 1024 * 1024 * 1024,
            last_updated: Utc::now(),
        }
    }

    /// Add a model to the manifest
    pub fn add_model(&mut self, model: CachedModel) {
        self.total_size_bytes += model.size_bytes;
        self.models.insert(model.model_id.clone(), model);
        self.last_updated = Utc::now();
    }

    /// Remove a model from the manifest
    pub fn remove_model(&mut self, model_id: &str) -> Option<CachedModel> {
        if let Some(model) = self.models.remove(model_id) {
            self.total_size_bytes = self.total_size_bytes.saturating_sub(model.size_bytes);
            self.last_updated = Utc::now();
            Some(model)
        } else {
            None
        }
    }

    /// Get a model from the manifest
    pub fn get_model(&self, model_id: &str) -> Option<&CachedModel> {
        self.models.get(model_id)
    }

    /// Clean up least-recently-used models if over size limit
    pub fn cleanup_lru(&mut self) -> Vec<String> {
        let mut removed = Vec::new();

        while self.total_size_bytes > self.max_cache_size_bytes && !self.models.is_empty() {
            // Find LRU model
            let lru_model_id = self
                .models
                .iter()
                .min_by_key(|(_, model)| model.last_accessed)
                .map(|(id, _)| id.clone());

            if let Some(model_id) = lru_model_id {
                self.remove_model(&model_id);
                removed.push(model_id);
            } else {
                break;
            }
        }

        removed
    }

    /// Validate integrity of all models
    pub fn validate_integrity(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut valid = Vec::new();
        let mut corrupted = Vec::new();
        let mut missing = Vec::new();

        for (model_id, model) in &self.models {
            if !model.path.exists() {
                missing.push(model_id.clone());
            } else if model.validate().is_err() {
                corrupted.push(model_id.clone());
            } else {
                valid.push(model_id.clone());
            }
        }

        (valid, corrupted, missing)
    }
}

/// User configuration with preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfiguration {
    pub default_shell: Option<ShellType>,
    pub safety_level: SafetyLevel,
    pub default_model: Option<String>,
    pub log_level: LogLevel,
    pub cache_max_size_gb: u64,
    pub log_rotation_days: u32,
}

impl Default for UserConfiguration {
    fn default() -> Self {
        Self {
            default_shell: None, // Auto-detect
            safety_level: SafetyLevel::Moderate,
            default_model: None,
            log_level: LogLevel::Info,
            cache_max_size_gb: 10,
            log_rotation_days: 7,
        }
    }
}

impl UserConfiguration {
    /// Create a builder for UserConfiguration
    pub fn builder() -> UserConfigurationBuilder {
        UserConfigurationBuilder::new()
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.cache_max_size_gb < 1 || self.cache_max_size_gb > 1000 {
            return Err(format!(
                "cache_max_size_gb must be between 1 and 1000, got {}",
                self.cache_max_size_gb
            ));
        }
        if self.log_rotation_days < 1 || self.log_rotation_days > 365 {
            return Err(format!(
                "log_rotation_days must be between 1 and 365, got {}",
                self.log_rotation_days
            ));
        }
        Ok(())
    }
}

/// Builder for UserConfiguration
pub struct UserConfigurationBuilder {
    default_shell: Option<ShellType>,
    safety_level: SafetyLevel,
    default_model: Option<String>,
    log_level: LogLevel,
    cache_max_size_gb: u64,
    log_rotation_days: u32,
}

impl Default for UserConfigurationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UserConfigurationBuilder {
    pub fn new() -> Self {
        let defaults = UserConfiguration::default();
        Self {
            default_shell: defaults.default_shell,
            safety_level: defaults.safety_level,
            default_model: defaults.default_model,
            log_level: defaults.log_level,
            cache_max_size_gb: defaults.cache_max_size_gb,
            log_rotation_days: defaults.log_rotation_days,
        }
    }

    pub fn default_shell(mut self, shell: ShellType) -> Self {
        self.default_shell = Some(shell);
        self
    }

    pub fn safety_level(mut self, level: SafetyLevel) -> Self {
        self.safety_level = level;
        self
    }

    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    pub fn cache_max_size_gb(mut self, size: u64) -> Self {
        self.cache_max_size_gb = size;
        self
    }

    pub fn log_rotation_days(mut self, days: u32) -> Self {
        self.log_rotation_days = days;
        self
    }

    pub fn build(self) -> Result<UserConfiguration, String> {
        let config = UserConfiguration {
            default_shell: self.default_shell,
            safety_level: self.safety_level,
            default_model: self.default_model,
            log_level: self.log_level,
            cache_max_size_gb: self.cache_max_size_gb,
            log_rotation_days: self.log_rotation_days,
        };
        config.validate()?;
        Ok(config)
    }
}

/// Configuration schema for validation
pub struct ConfigSchema {
    pub known_sections: Vec<String>,
    pub known_keys: HashMap<String, String>,
    pub deprecated_keys: HashMap<String, String>,
}

impl ConfigSchema {
    pub fn new() -> Self {
        let mut known_keys = HashMap::new();
        known_keys.insert(
            "general.safety_level".to_string(),
            "SafetyLevel enum".to_string(),
        );
        known_keys.insert(
            "general.default_shell".to_string(),
            "ShellType enum".to_string(),
        );
        known_keys.insert("general.default_model".to_string(), "String".to_string());
        known_keys.insert("logging.log_level".to_string(), "LogLevel enum".to_string());
        known_keys.insert("logging.log_rotation_days".to_string(), "u32".to_string());
        known_keys.insert("cache.max_size_gb".to_string(), "u64".to_string());

        Self {
            known_sections: vec![
                "general".to_string(),
                "logging".to_string(),
                "cache".to_string(),
            ],
            known_keys,
            deprecated_keys: HashMap::new(),
        }
    }

    pub fn validate(&self, _config: &UserConfiguration) -> Result<(), String> {
        // Validation is done in UserConfiguration::validate()
        Ok(())
    }
}

/// Execution context captured at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub current_dir: PathBuf,
    pub shell_type: ShellType,
    pub platform: Platform,
    pub environment_vars: HashMap<String, String>,
    pub username: String,
    pub hostname: String,
    pub captured_at: DateTime<Utc>,
}

impl ExecutionContext {
    /// Create new execution context with custom values
    pub fn new(
        current_dir: PathBuf,
        shell_type: ShellType,
        platform: Platform,
    ) -> Result<Self, String> {
        if !current_dir.is_absolute() {
            return Err("Current directory must be absolute path".to_string());
        }

        // Capture and filter environment variables
        let environment_vars = Self::filter_env_vars();

        Ok(Self {
            current_dir,
            shell_type,
            platform,
            environment_vars,
            username: std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "unknown".to_string()),
            hostname: std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("COMPUTERNAME"))
                .unwrap_or_else(|_| "unknown".to_string()),
            captured_at: Utc::now(),
        })
    }

    /// Filter environment variables to exclude sensitive data
    fn filter_env_vars() -> HashMap<String, String> {
        let sensitive_patterns = [
            "API_KEY",
            "TOKEN",
            "SECRET",
            "PASSWORD",
            "PASSWD",
            "CREDENTIAL",
            "AUTH",
            "PRIVATE",
            "KEY",
        ];

        std::env::vars()
            .filter(|(key, value)| {
                // Filter out sensitive variables and empty values
                !key.is_empty()
                    && !value.is_empty()
                    && !sensitive_patterns
                        .iter()
                        .any(|pattern| key.to_uppercase().contains(pattern))
            })
            .collect()
    }

    /// Serialize context for LLM prompt
    pub fn to_prompt_context(&self) -> String {
        format!(
            "Current directory: {}\nShell: {}\nPlatform: {}\nUser: {}@{}",
            self.current_dir.display(),
            self.shell_type,
            self.platform,
            self.username,
            self.hostname
        )
    }

    /// Check if environment variable exists
    pub fn has_env_var(&self, key: &str) -> bool {
        self.environment_vars.contains_key(key)
    }

    /// Get environment variable value
    pub fn get_env_var(&self, key: &str) -> Option<&str> {
        self.environment_vars.get(key).map(|s| s.as_str())
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub operation_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub duration_ms: Option<u64>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            target: target.into(),
            message: message.into(),
            operation_id: None,
            metadata: HashMap::new(),
            duration_ms: None,
        }
    }

    /// Add metadata field
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set operation ID
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}
