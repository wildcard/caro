//! Manual Testing Infrastructure
//!
//! Provides interactive testing capabilities for the cmdai safety validation system.
//! Enables users to run structured test cases, inspect logs, analyze results,
//! and iterate on improvements in real-time.

pub mod runner;
pub mod cases;
pub mod analyzer;
pub mod reporter;
pub mod metrics;

pub use runner::ManualTestRunner;
pub use cases::{TestCase, TestCategory, TestResult, TestOutcome};
pub use analyzer::LogAnalyzer;
pub use reporter::InteractiveReporter;
pub use metrics::TestMetrics;