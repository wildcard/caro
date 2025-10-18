//! cmdai - Natural Language to Shell Command CLI Tool
//!
//! This library provides core functionality for converting natural language
//! descriptions into safe, POSIX-compliant shell commands using local LLMs.
//!
//! # Core Modules
//!
//! - [`models`] - Core data types (CommandRequest, GeneratedCommand, enums)
//! - [`safety`] - Safety validation with dangerous command detection
//! - [`backends`] - Command generation backends (Embedded, Ollama, vLLM)
//! - [`cli`] - CLI interface and argument parsing
//! - [`cache`] - Model caching with integrity validation
//! - [`config`] - Configuration management with TOML support
//! - [`execution`] - Execution context capture and shell detection
//! - [`logging`] - Structured logging with sensitive data redaction
//!
//! # Example
//!
//! ```no_run
//! use cmdai::models::{CommandRequest, ShellType, SafetyLevel};
//!
//! let request = CommandRequest::new("list all files", ShellType::Bash)
//!     .with_safety(SafetyLevel::Moderate);
//! ```

pub mod backends;
pub mod cache;
pub mod cli;
pub mod config;
pub mod execution;
pub mod history;
pub mod logging;
pub mod model_loader;
pub mod models;
pub mod performance;
pub mod safety;
pub mod semantic;
pub mod streaming;

// Re-export commonly used types for convenience
pub use models::{
    BackendInfo, BackendType, CacheManifest, CachedModel, CommandRequest, ConfigSchema,
    ExecutionContext, GeneratedCommand, LogEntry, LogLevel, Platform, RiskLevel, SafetyLevel,
    ShellType, UserConfiguration, UserConfigurationBuilder,
};

// Re-export infrastructure module types and errors
pub use cache::{CacheError, CacheManager, CacheStats, IntegrityReport};
pub use config::{ConfigError, ConfigManager};
pub use execution::{ExecutionError, PlatformDetector, ShellDetector};
pub use logging::{LogConfig, LogConfigBuilder, LogError, LogFormat, LogOutput, Logger, Redaction};
pub use model_loader::{ModelInfo, ModelLoader};

// Advanced safety validation types
pub use safety::advanced::{
    AdvancedSafetyConfig, AdvancedSafetyValidator, AdvancedValidationResult, BehavioralPattern,
    ExecutionStats, SystemMetrics, ThreatLevel, UserFeedback, UserPrivileges, ValidationContext,
};

// Streaming command generation types
pub use streaming::{
    CancellationToken, StreamChunk, StreamingCommandGenerator, StreamingConfig, StreamingError,
    StreamingGenerator, StreamingStats, StreamingWrapper,
};

// Performance monitoring types
pub use performance::{
    BackendMetrics, HealthStatus, MetricsCollector, PerformanceMonitor, PerformanceSnapshot,
    RealTimeStats, RingBuffer, SelectionStrategy,
};

// Semantic search and embedding types
pub use semantic::cache::{
    CacheCleanupPolicy, CacheEntry, CacheStatistics, EmbeddingCacheError, EmbeddingMetadata,
    LocalEmbeddingCache,
};

// Re-export backend types
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub use backends::embedded::MlxBackend;
pub use backends::embedded::{
    CpuBackend, EmbeddedConfig, EmbeddedModelBackend, InferenceBackend, ModelVariant,
};
#[cfg(feature = "remote-backends")]
pub use backends::remote::{OllamaBackend, VllmBackend};
pub use backends::selector::{BackendSelector, BackendSelectorConfig, SmartBackend};
pub use backends::{BackendInfo as BackendInfoTrait, CommandGenerator, GeneratorError};
