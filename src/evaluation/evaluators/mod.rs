//! Evaluator trait and implementations for test categories
//!
//! This module contains the core Evaluator trait and implementations
//! for each test category: Correctness, Safety, POSIX, and Multi-backend.

use async_trait::async_trait;
use crate::evaluation::{EvaluationResult, TestCase, TestCategory};
use crate::evaluation::errors::Result;

/// Result of a backend command generation attempt
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// The generated command (None if generation failed or was blocked)
    pub command: Option<String>,

    /// Whether the command was blocked by safety validation
    pub blocked: bool,

    /// Error message if generation failed
    pub error: Option<String>,

    /// Time taken to generate command (milliseconds)
    pub execution_time_ms: u64,

    /// Backend that generated this result
    pub backend_name: String,
}

impl CommandResult {
    /// Creates a successful command generation result
    pub fn success(command: String, execution_time_ms: u64, backend_name: String) -> Self {
        Self {
            command: Some(command),
            blocked: false,
            error: None,
            execution_time_ms,
            backend_name,
        }
    }

    /// Creates a blocked command result (safety violation)
    pub fn blocked(execution_time_ms: u64, backend_name: String) -> Self {
        Self {
            command: None,
            blocked: true,
            error: None,
            execution_time_ms,
            backend_name,
        }
    }

    /// Creates a failed command generation result
    pub fn failed(error: String, execution_time_ms: u64, backend_name: String) -> Self {
        Self {
            command: None,
            blocked: false,
            error: Some(error),
            execution_time_ms,
            backend_name,
        }
    }
}

/// Core trait for all evaluation categories
///
/// Each category (Correctness, Safety, POSIX, Multi-backend) implements
/// this trait to provide category-specific validation logic.
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// Returns the test category this evaluator handles
    fn category(&self) -> TestCategory;

    /// Evaluates a test case result
    ///
    /// # Arguments
    ///
    /// * `test_case` - The test case being evaluated
    /// * `result` - The command generation result from the backend
    ///
    /// # Returns
    ///
    /// An EvaluationResult indicating pass/fail with details
    ///
    /// # Contract
    ///
    /// - Must be deterministic for the same inputs
    /// - Must complete within 5 seconds
    /// - Errors must be descriptive and actionable
    async fn evaluate(
        &self,
        test_case: &TestCase,
        result: &CommandResult,
    ) -> Result<EvaluationResult>;
}

// Sub-modules for specific evaluator implementations
pub mod correctness;
pub mod safety;
pub mod posix;
pub mod consistency;
pub mod utils;

// Re-exports for public API
pub use correctness::CorrectnessEvaluator;
pub use safety::SafetyEvaluator;
pub use posix::POSIXEvaluator;
pub use consistency::ConsistencyEvaluator;
