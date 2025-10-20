// Core data types and traits for cmdai evaluation framework

pub mod dataset;
pub mod results;
pub mod types;

// Re-export commonly used types
pub use dataset::{DatasetStats, TestCase, TestDataset};
pub use results::{
    AssertionFailure, CategoryResult, CommandAccuracy, EvaluationResult, PerformanceMetrics,
    RuntimeResult, RuntimeStats, TestCaseResult,
};
pub use types::{
    AssertionConfig, CommandStringAssertions, DifficultyLevel, DockerConfig, FileExpectation,
    RuntimeAssertions, SafetyLevel, SandboxBackend, SandboxConfig, ShellType,
};
