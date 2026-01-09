//! Error types for the LLM Evaluation Harness
//!
//! This module defines all error types that can occur during evaluation,
//! using the `thiserror` crate for ergonomic error handling.

use std::io;
use thiserror::Error;

/// Main error type for evaluation operations
#[derive(Debug, Error)]
pub enum EvaluationError {
    /// Dataset loading or parsing error
    #[error("Dataset error: {0}")]
    Dataset(#[from] DatasetError),

    /// Test case validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Backend execution error
    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),

    /// Baseline comparison error
    #[error("Baseline error: {0}")]
    Baseline(#[from] BaselineError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    /// Generic error with context
    #[error("Evaluation failed: {0}")]
    Other(String),
}

/// Dataset-related errors
#[derive(Debug, Error)]
pub enum DatasetError {
    /// File not found
    #[error("Dataset file not found: {path}")]
    FileNotFound { path: String },

    /// YAML parsing error
    #[error("Failed to parse YAML dataset: {source}")]
    YamlParse {
        #[from]
        source: serde_yaml::Error,
    },

    /// JSON parsing error (for baselines)
    #[error("Failed to parse JSON data: {source}")]
    JsonParse {
        #[from]
        source: serde_json::Error,
    },

    /// Invalid dataset structure
    #[error("Invalid dataset structure: {reason}")]
    InvalidStructure { reason: String },

    /// Empty dataset
    #[error("Dataset is empty")]
    Empty,

    /// Duplicate test IDs
    #[error("Duplicate test ID found: {id}")]
    DuplicateId { id: String },

    /// Test case validation failed
    #[error("Test case '{id}' validation failed: {reason}")]
    InvalidTestCase { id: String, reason: String },

    /// Missing required field
    #[error("Missing required field '{field}' in test case '{id}'")]
    MissingField { id: String, field: String },

    /// Invalid category distribution
    #[error("Invalid category distribution: {reason}")]
    InvalidDistribution { reason: String },
}

/// Backend execution errors
#[derive(Debug, Error)]
pub enum BackendError {
    /// Backend not available
    #[error("Backend '{name}' is not available")]
    NotAvailable { name: String },

    /// Backend initialization failed
    #[error("Failed to initialize backend '{name}': {reason}")]
    InitializationFailed { name: String, reason: String },

    /// Command generation failed
    #[error("Backend '{backend}' failed to generate command for test '{test_id}': {reason}")]
    GenerationFailed {
        backend: String,
        test_id: String,
        reason: String,
    },

    /// Backend timeout
    #[error("Backend '{backend}' timed out on test '{test_id}' after {timeout_ms}ms")]
    Timeout {
        backend: String,
        test_id: String,
        timeout_ms: u64,
    },

    /// Invalid backend configuration
    #[error("Invalid backend configuration for '{name}': {reason}")]
    InvalidConfig { name: String, reason: String },

    /// Backend feature not available on platform
    #[error("Backend '{name}' requires features not available: {missing_features:?}")]
    MissingPlatformFeatures {
        name: String,
        missing_features: Vec<String>,
    },
}

/// Baseline comparison and storage errors
#[derive(Debug, Error)]
pub enum BaselineError {
    /// Baseline file not found
    #[error("Baseline file not found: {path}")]
    FileNotFound { path: String },

    /// Failed to load baseline
    #[error("Failed to load baseline from '{path}': {reason}")]
    LoadFailed { path: String, reason: String },

    /// Failed to save baseline
    #[error("Failed to save baseline to '{path}': {reason}")]
    SaveFailed { path: String, reason: String },

    /// Incompatible baseline format
    #[error("Baseline format incompatible: expected version {expected}, found {actual}")]
    IncompatibleFormat { expected: String, actual: String },

    /// Missing baseline for comparison
    #[error("No baseline available for comparison")]
    NoBaseline,

    /// Invalid comparison (different test sets)
    #[error("Cannot compare: baseline has {baseline_tests} tests, current has {current_tests} tests")]
    IncompatibleTestSets {
        baseline_tests: usize,
        current_tests: usize,
    },

    /// Regression detected
    #[error("Regression detected: pass rate dropped by {delta:.1}% (threshold: {threshold:.1}%)")]
    RegressionDetected { delta: f32, threshold: f32 },
}

/// Serialization/deserialization errors
#[derive(Debug, Error)]
pub enum SerializationError {
    /// JSON serialization failed
    #[error("JSON serialization failed: {source}")]
    JsonSerialize {
        #[from]
        source: serde_json::Error,
    },

    /// YAML serialization failed
    #[error("YAML serialization failed: {source}")]
    YamlSerialize {
        #[from]
        source: serde_yaml::Error,
    },

    /// Invalid format
    #[error("Invalid format: {reason}")]
    InvalidFormat { reason: String },
}

/// Result type alias for evaluation operations
pub type Result<T> = std::result::Result<T, EvaluationError>;

/// Result type alias for dataset operations
pub type DatasetResult<T> = std::result::Result<T, DatasetError>;

/// Result type alias for baseline operations
pub type BaselineResult<T> = std::result::Result<T, BaselineError>;

impl EvaluationError {
    /// Creates a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        EvaluationError::Validation(msg.into())
    }

    /// Creates a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        EvaluationError::Config(msg.into())
    }

    /// Creates a generic error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        EvaluationError::Other(msg.into())
    }
}

impl DatasetError {
    /// Creates a test case validation error
    pub fn invalid_test_case<S1, S2>(id: S1, reason: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        DatasetError::InvalidTestCase {
            id: id.into(),
            reason: reason.into(),
        }
    }

    /// Creates a duplicate ID error
    pub fn duplicate_id<S: Into<String>>(id: S) -> Self {
        DatasetError::DuplicateId { id: id.into() }
    }
}

impl BackendError {
    /// Creates a generation failed error
    pub fn generation_failed<S1, S2, S3>(backend: S1, test_id: S2, reason: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        BackendError::GenerationFailed {
            backend: backend.into(),
            test_id: test_id.into(),
            reason: reason.into(),
        }
    }

    /// Creates a timeout error
    pub fn timeout<S1, S2>(backend: S1, test_id: S2, timeout_ms: u64) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        BackendError::Timeout {
            backend: backend.into(),
            test_id: test_id.into(),
            timeout_ms,
        }
    }
}

impl BaselineError {
    /// Creates a regression detected error
    pub fn regression(delta: f32, threshold: f32) -> Self {
        BaselineError::RegressionDetected { delta, threshold }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EvaluationError::validation("Invalid input");
        assert_eq!(err.to_string(), "Validation error: Invalid input");
    }

    #[test]
    fn test_dataset_error_conversion() {
        let dataset_err = DatasetError::Empty;
        let eval_err: EvaluationError = dataset_err.into();
        assert!(matches!(eval_err, EvaluationError::Dataset(_)));
    }

    #[test]
    fn test_backend_error_helpers() {
        let err = BackendError::timeout("mlx", "test-001", 5000);
        assert_eq!(
            err.to_string(),
            "Backend 'mlx' timed out on test 'test-001' after 5000ms"
        );
    }

    #[test]
    fn test_baseline_error_regression() {
        let err = BaselineError::regression(-5.5, 5.0);
        assert!(err.to_string().contains("-5.5%"));
        assert!(err.to_string().contains("5.0%"));
    }
}
