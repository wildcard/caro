//! Caro LLM Evaluation Harness
//!
//! This harness provides comprehensive evaluation of LLM-generated shell command quality
//! across correctness, safety, and POSIX compliance dimensions.
//!
//! # Modules
//!
//! - `dataset`: Test dataset loading and validation
//! - `executor`: CLI invocation and result capture
//! - `evaluator`: Command correctness scoring
//! - `safety_validator`: Safety pattern detection validation
//! - `posix_checker`: POSIX compliance checking with shellcheck
//! - `reporter`: JSON and Markdown report generation
//!
//! # Usage
//!
//! ```ignore
//! use caro_evaluation::dataset::TestDataset;
//! use caro_evaluation::executor::Executor;
//! use caro_evaluation::evaluator::Evaluator;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let dataset = TestDataset::load_from_file("datasets/correctness/file_operations.json".as_ref())?;
//! let executor = Executor::new()?;
//! let evaluator = Evaluator;
//!
//! for test_case in &dataset.test_cases {
//!     let generated = executor.execute(&test_case.prompt).await?;
//!     let result = evaluator.evaluate_correctness(&generated, &test_case.expected_command);
//!     println!("{}: {:.2}%", test_case.id, result.score * 100.0);
//! }
//! # Ok(())
//! # }
//! ```

pub mod capability_matrix;
pub mod dashboard;
pub mod dataset;
pub mod dataset_export;
pub mod evaluator;
pub mod executor;
pub mod issue_automation;
pub mod model_profiling;
pub mod pattern_extraction;
pub mod posix_checker;
pub mod prompt_comparison;
pub mod prompts;
pub mod reporter;
pub mod safety_validator;
pub mod test_runner;
pub mod timeseries;
pub mod training_tracker;
