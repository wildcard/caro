//! Command line of the `harness = false` evaluation target.
//!
//! libtest filter handling (#1162):
//! `cargo test <filter>` passes `<filter>` to every test binary. libtest runs
//! only the tests whose names contain it. This target is one "test" named
//! `evaluation`, so it runs when the filter is a substring of that name and
//! skips itself otherwise.

use clap::Parser;
use std::path::PathBuf;

/// CLI arguments for evaluation harness
#[derive(Parser, Debug)]
#[command(name = "evaluation")]
#[command(about = "Run LLM evaluation harness", long_about = None)]
pub struct Args {
    /// Test category to run (correctness, safety, posix, multi_backend, execution)
    #[arg(long)]
    pub category: Option<String>,

    /// Dataset file to load (YAML or JSON test-case array)
    #[arg(long, default_value = "tests/evaluation/dataset.yaml")]
    pub dataset: PathBuf,

    /// Sandbox tier for execution-category cases (off, tier0).
    /// tier0 = just-bash via tools/exec-harness (needs `npm ci` there);
    /// when unavailable, execution cases are skipped, never failed.
    #[arg(long, default_value = "off")]
    pub execution_tier: String,

    /// Backend to test (static_matcher, mlx, ollama, vllm)
    #[arg(long)]
    pub backend: Option<String>,

    /// Output format (json or table)
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Path to baseline JSON for comparison
    #[arg(long)]
    pub baseline: Option<PathBuf>,

    /// Regression threshold (default: 0.05 for 5%)
    #[arg(long, default_value = "0.05")]
    pub threshold: f32,

    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,

    /// libtest name filter, passed by `cargo test <filter>` (#1162)
    #[arg(hide = true)]
    pub filter: Option<String>,
}

/// Name that a `cargo test <filter>` filter is matched against.
pub const TARGET_NAME: &str = "evaluation";

/// Whether the evaluation should run for the libtest filter `filter`.
pub fn should_run(filter: Option<&str>) -> bool {
    filter.is_none_or(|f| TARGET_NAME.contains(f))
}
