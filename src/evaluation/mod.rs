//! LLM Evaluation Harness
//!
//! Comprehensive testing framework for shell command generation quality, safety,
//! and correctness across multiple LLM backends.
//!
//! ## Architecture
//!
//! The evaluation harness consists of several key components:
//!
//! - **Models**: Core data structures (`TestCase`, `BenchmarkReport`, etc.)
//! - **Dataset**: YAML-based test case storage and loading
//! - **Evaluators**: Trait-based evaluation implementations for each category
//! - **Harness**: Orchestration and parallel execution
//! - **Baseline**: Regression detection and comparison
//!
//! ## Test Categories
//!
//! 1. **Correctness**: Validates functional accuracy of generated commands
//! 2. **Safety**: Ensures dangerous patterns are blocked appropriately
//! 3. **POSIX**: Verifies cross-platform compatibility
//! 4. **Multi-backend**: Tests consistency across different LLM backends
//!
//! ## Usage
//!
//! ```rust,no_run
//! use caro::evaluation::{Dataset, EvaluationHarness, Config};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load test dataset
//! let dataset = Dataset::load("tests/evaluation/dataset.yaml").await?;
//!
//! // Configure harness
//! let config = Config::default();
//! let harness = EvaluationHarness::new(config);
//!
//! // Run evaluation
//! let report = harness.evaluate(&dataset).await?;
//! println!("Pass rate: {:.1}%", report.overall_pass_rate * 100.0);
//! # Ok(())
//! # }
//! ```
//!
//! ## Integration with CI/CD
//!
//! The harness integrates with `cargo test` for familiar developer workflows:
//!
//! ```bash
//! # Run full evaluation suite
//! cargo test --test evaluation
//!
//! # Filter by category
//! cargo test --test evaluation -- --category safety
//!
//! # Compare against baseline
//! cargo test --test evaluation -- --baseline tests/evaluation/baselines/main-latest.json
//! ```

// Sub-modules
pub mod dataset;
pub mod errors;
pub mod evaluators;
pub mod models;
// pub mod harness;
// pub mod baseline;

// Re-exports for public API
pub use dataset::*;
pub use errors::*;
pub use evaluators::{CommandResult, Evaluator};
pub use models::*;
