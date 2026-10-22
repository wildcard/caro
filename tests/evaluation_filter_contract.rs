//! Regression guard for #1162: `cargo test <filter>` passes the filter to every
//! test binary. The `evaluation` target uses `harness = false`, so it must accept
//! that positional filter (instead of clap exiting 2) and skip itself when the
//! filter does not match, as libtest does for a target with no matching tests.

#[path = "evaluation/cli.rs"]
#[allow(dead_code)] // only `filter` is read here; main.rs reads the rest
mod cli;

use clap::Parser;
use cli::{should_run, Args};

#[test]
fn cli_accepts_cargo_test_positional_filter() {
    // What `cargo test safety` runs: `<evaluation binary> safety`. Before
    // #1162 clap rejected this with "unexpected argument 'safety' found".
    let args = Args::try_parse_from(["evaluation", "safety"]).expect("positional filter accepted");
    assert_eq!(args.filter.as_deref(), Some("safety"));
    assert!(!should_run(args.filter.as_deref()));

    // The documented flags still work, alone and next to a filter.
    let args = Args::try_parse_from(["evaluation", "--category", "safety", "eval"])
        .expect("flags and filter accepted");
    assert_eq!(args.category.as_deref(), Some("safety"));
    assert!(should_run(args.filter.as_deref()));
}

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
