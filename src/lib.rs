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
//! - [`intelligence`] - Context intelligence engine (V2 Phase 1)
//! - [`learning`] - Learning engine for pattern storage and command explanation (V2 Phase 1)
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
pub mod intelligence;
pub mod learning;
pub mod logging;
pub mod model_loader;
pub mod models;
pub mod platform;
pub mod safety;

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
pub use model_loader::ModelLoader;

// Re-export backend types
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub use backends::embedded::MlxBackend;
pub use backends::embedded::{
    CpuBackend, EmbeddedConfig, EmbeddedModelBackend, InferenceBackend, ModelVariant,
};
#[cfg(feature = "remote-backends")]
pub use backends::remote::{OllamaBackend, VllmBackend};
pub use backends::{BackendInfo as BackendInfoTrait, CommandGenerator, GeneratorError};

// Re-export intelligence types (V2 Phase 1)
pub use intelligence::{
    ContextError, ContextGraph, ContextOptions, EnvironmentContext, GitAnalyzer, GitContext,
    HistoryAnalyzer, HistoryContext, InfrastructureContext, ProjectContext, ProjectParser,
    ProjectType, Tool, ToolDetector,
};

// Re-export learning types (V2 Phase 1)
pub use learning::{
    Achievement, AchievementTracker, Alternative, CommandExplainer, CommandInfo, CommandPattern,
    Difficulty, Example, Explanation, ExplanationPart, ImprovementLearner, ImprovementPattern,
    LearningConfig, LearningEngine, LearningStats, Lesson, PatternDB, Quiz, SimilaritySearch,
    Tutorial, TutorialResult, UnlockCondition,
};

// Re-export platform types
pub use platform::{
    init as platform_init, Architecture, OperatingSystem, PlatformInfo, Shell as PlatformShell,
};
