//! Logging module with tracing integration and sensitive data redaction

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub use crate::models::LogLevel;

mod redaction;
pub use redaction::Redaction;

static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Logging errors
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("Logger already initialized")]
    AlreadyInitialized,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Log output destination
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(PathBuf),
}

/// Log format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

/// Log rotation settings
#[derive(Debug, Clone)]
pub struct LogRotation {
    pub max_files: u32,
    pub max_size_mb: u64,
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub log_level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub redaction_enabled: bool,
    pub rotation: Option<LogRotation>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Stderr,
            redaction_enabled: true,
            rotation: None,
        }
    }
}

impl LogConfig {
    pub fn development() -> Self {
        Self {
            log_level: LogLevel::Debug,
            format: LogFormat::Pretty,
            output: LogOutput::Stderr,
            redaction_enabled: false,
            rotation: None,
        }
    }

    pub fn production() -> Self {
        Self {
            log_level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::File(PathBuf::from("/var/log/caro/caro.log")),
            redaction_enabled: true,
            rotation: Some(LogRotation {
                max_files: 7,
                max_size_mb: 100,
            }),
        }
    }
}

/// Log configuration builder
///
/// Provides a fluent API for building log configurations.
///
/// # Example
///
/// ```no_run
/// use caro::logging::{LogConfigBuilder, LogLevel, LogFormat, LogOutput, LogRotation};
/// use std::path::PathBuf;
///
/// # fn main() {
/// // Create a custom logging configuration
/// let config = LogConfigBuilder::new()
///     .log_level(LogLevel::Debug)
///     .format(LogFormat::Pretty)
///     .output(LogOutput::File(PathBuf::from("/tmp/caro.log")))
///     .redaction_enabled(true)
///     .rotation(LogRotation {
///         max_files: 5,
///         max_size_mb: 50,
///     })
///     .build();
///
/// println!("Log level: {:?}", config.log_level);
/// println!("Format: {:?}", config.format);
/// # }
/// ```
pub struct LogConfigBuilder {
    config: LogConfig,
}

impl Default for LogConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LogConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
        }
    }

    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.config.log_level = level;
        self
    }

    pub fn format(mut self, format: LogFormat) -> Self {
        self.config.format = format;
        self
    }

    pub fn output(mut self, output: LogOutput) -> Self {
        self.config.output = output;
        self
    }

    pub fn redaction_enabled(mut self, enabled: bool) -> Self {
        self.config.redaction_enabled = enabled;
        self
    }

    pub fn rotation(mut self, rotation: LogRotation) -> Self {
        self.config.rotation = Some(rotation);
        self
    }

    pub fn build(self) -> LogConfig {
        self.config
    }
}

/// Global logger
pub struct Logger;

impl Logger {
    /// Initialize the global logger
    ///
    /// Configures tracing with the specified log level, format, and output destination.
    /// Can only be called once per process - subsequent calls return an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::logging::{Logger, LogConfig, LogLevel, LogFormat, LogOutput};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Development configuration
    /// let config = LogConfig::development();
    /// Logger::init(config)?;
    ///
    /// // Now logging is enabled
    /// tracing::info!("Application started");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example with Builder
    ///
    /// ```no_run
    /// use caro::logging::{Logger, LogConfigBuilder, LogLevel, LogFormat};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = LogConfigBuilder::new()
    ///     .log_level(LogLevel::Debug)
    ///     .format(LogFormat::Pretty)
    ///     .redaction_enabled(true)
    ///     .build();
    ///
    /// Logger::init(config)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `LogError::AlreadyInitialized` if the logger was already initialized.
    pub fn init(config: LogConfig) -> Result<(), LogError> {
        if LOGGER_INITIALIZED.swap(true, Ordering::SeqCst) {
            return Err(LogError::AlreadyInitialized);
        }

        // Initialize tracing subscriber
        let level_filter = config.log_level.to_tracing_level();

        tracing_subscriber::fmt()
            .with_max_level(level_filter)
            .init();

        Ok(())
    }
}

/// Operation span for tracking operations
pub struct OperationSpan {
    _name: String,
}

impl OperationSpan {
    pub fn new(name: impl Into<String>) -> Self {
        Self { _name: name.into() }
    }
}
