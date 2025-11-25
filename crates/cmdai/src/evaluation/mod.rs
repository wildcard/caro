// Command accuracy evaluation framework

pub mod dataset;
pub mod engine;
pub mod metrics;
pub mod runner;

pub use dataset::{TestCase, TestDataset, DifficultyLevel, SafetyLevel, DatasetStats};
pub use engine::EvaluationEngine;
pub use metrics::{AccuracyScore, EvaluationResult, PerformanceMetrics, TestCaseResult};
pub use runner::EvaluationRunner;