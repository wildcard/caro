//! libtest filter handling for the `harness = false` evaluation target (#1162).
//!
//! `cargo test <filter>` passes `<filter>` to every test binary. libtest runs
//! only the tests whose names contain it. This target is one "test" named
//! `evaluation`, so it runs when the filter is a substring of that name and
//! skips itself otherwise.

/// Name that a `cargo test <filter>` filter is matched against.
pub const TARGET_NAME: &str = "evaluation";

/// Whether the evaluation should run for the libtest filter `filter`.
pub fn should_run(filter: Option<&str>) -> bool {
    filter.is_none_or(|f| TARGET_NAME.contains(f))
}
