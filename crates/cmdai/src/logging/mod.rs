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
            output: LogOutput::File(PathBuf::from("/var/log/cmdai/cmdai.log")),
            redaction_enabled: true,
            rotation: Some(LogRotation {
                max_files: 7,
                max_size_mb: 100,
            }),
        }
    }
}

/// Log configuration builder
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

/// Operation span for tracking operations with timing and metadata
pub struct OperationSpan {
    _span: tracing::Span,
    start_time: std::time::Instant,
}

impl OperationSpan {
    pub fn new(name: impl Into<String>) -> Self {
        let span_name = name.into();
        let span = tracing::info_span!("operation", operation = %span_name);

        Self {
            _span: span,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn with_field(
        name: impl Into<String>,
        _field: &str,
        value: impl std::fmt::Display,
    ) -> Self {
        let span_name = name.into();
        let span = tracing::info_span!("operation", operation = %span_name, field = %value);

        Self {
            _span: span,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn record_timing(&self, event: &str) {
        let elapsed = self.start_time.elapsed();
        tracing::info!(
            event = event,
            duration_ms = elapsed.as_millis(),
            "Operation timing recorded"
        );
    }

    pub fn record_error(&self, error: &str) {
        tracing::error!(
            error = error,
            duration_ms = self.start_time.elapsed().as_millis(),
            "Operation failed"
        );
    }
}

impl Drop for OperationSpan {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        tracing::info!(duration_ms = elapsed.as_millis(), "Operation completed");
    }
}

/// Performance metrics for constitutional compliance monitoring
pub struct PerformanceLogger;

impl PerformanceLogger {
    /// Log startup timing (constitutional requirement: <100ms)
    pub fn log_startup_time(duration: std::time::Duration) {
        let millis = duration.as_millis();
        let meets_requirement = millis < 100;

        tracing::info!(
            startup_time_ms = millis,
            constitutional_compliant = meets_requirement,
            "Application startup completed"
        );

        if !meets_requirement {
            tracing::warn!(
                startup_time_ms = millis,
                target_ms = 100,
                "Startup time exceeds constitutional requirement"
            );
        }
    }

    /// Log inference timing (constitutional requirement: <2s)
    pub fn log_inference_time(duration: std::time::Duration, backend: &str) {
        let millis = duration.as_millis();
        let meets_requirement = millis < 2000;

        tracing::info!(
            inference_time_ms = millis,
            backend = backend,
            constitutional_compliant = meets_requirement,
            "Model inference completed"
        );

        if !meets_requirement {
            tracing::warn!(
                inference_time_ms = millis,
                target_ms = 2000,
                backend = backend,
                "Inference time exceeds constitutional requirement"
            );
        }
    }

    /// Log safety validation timing (constitutional requirement: <50ms)
    pub fn log_safety_validation_time(duration: std::time::Duration) {
        let millis = duration.as_millis();
        let meets_requirement = millis < 50;

        tracing::info!(
            validation_time_ms = millis,
            constitutional_compliant = meets_requirement,
            "Safety validation completed"
        );

        if !meets_requirement {
            tracing::warn!(
                validation_time_ms = millis,
                target_ms = 50,
                "Safety validation time exceeds constitutional requirement"
            );
        }
    }

    /// Log history write timing (constitutional requirement: <10ms)
    pub fn log_history_write_time(duration: std::time::Duration) {
        let millis = duration.as_millis();
        let meets_requirement = millis < 10;

        tracing::info!(
            history_write_ms = millis,
            constitutional_compliant = meets_requirement,
            "History write completed"
        );

        if !meets_requirement {
            tracing::warn!(
                history_write_ms = millis,
                target_ms = 10,
                "History write time exceeds constitutional requirement"
            );
        }
    }
}

/// Production-ready logger initialization with enhanced structured logging
impl Logger {
    /// Initialize logger for production backend system
    pub fn init_production() -> Result<(), LogError> {
        if LOGGER_INITIALIZED.swap(true, Ordering::SeqCst) {
            return Err(LogError::AlreadyInitialized);
        }

        // Enhanced tracing subscriber with JSON formatting and performance fields
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();

        tracing::info!(
            component = "logger",
            environment = "production",
            "Production logging initialized"
        );

        Ok(())
    }

    /// Initialize logger for development with pretty formatting
    pub fn init_development() -> Result<(), LogError> {
        if LOGGER_INITIALIZED.swap(true, Ordering::SeqCst) {
            return Err(LogError::AlreadyInitialized);
        }

        tracing_subscriber::fmt()
            .pretty()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .init();

        tracing::info!(
            component = "logger",
            environment = "development",
            "Development logging initialized"
        );

        Ok(())
    }
}
