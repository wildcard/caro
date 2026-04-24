//! Dogma — shared rule-engine types.
//!
//! This module is the designated home of the YAML-rule compiler shared
//! between the CVE pipeline (spec 010) and the community rule engine
//! (spec 006, design-only). It exports only the serializable runtime
//! types; the compile-time helpers live in `compiler.rs` and are
//! consumed by `build.rs` via `#[path]` include.
//!
//! For the CVE pipeline, the compiled blob is deserialized by
//! `crate::safety::cve_patterns::CVE_COMPILED`.

pub mod compiler;

pub use compiler::{CompiledPattern, CompiledRuleset, RulesetMetadata};
