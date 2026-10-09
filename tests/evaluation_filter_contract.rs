//! Regression guard for #1162: `cargo test <filter>` passes the filter to every
//! test binary. The `evaluation` target uses `harness = false`, so it must accept
//! that positional filter (instead of clap exiting 2) and skip itself when the
//! filter does not match, as libtest does for a target with no matching tests.

#[path = "evaluation/libtest_filter.rs"]
mod libtest_filter;

use libtest_filter::should_run;

#[test]
fn no_filter_runs_the_evaluation() {
    assert!(should_run(None));
}

#[test]
fn unrelated_filter_skips_the_evaluation() {
    // `cargo test safety` (documented in CLAUDE.md) must not run or fail here.
    assert!(!should_run(Some("safety")));
    assert!(!should_run(Some("static_matcher")));
}

#[test]
fn filter_matching_the_target_name_runs_the_evaluation() {
    assert!(should_run(Some("evaluation")));
    assert!(should_run(Some("eval")));
}
