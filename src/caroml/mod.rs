//! CaroML — A meta-language for intent-tracked shell tasks.
//!
//! CaroML lets a human commit *intent* (a `.caro` file written in a small
//! line-keyword DSL) and have Caro deterministically (re)generate the
//! shell script that fulfills it. The lock (`.caro.lock`) records every
//! generation per platform with a track record so the file evolves with
//! models, CVE feeds, and accumulated team preference.
//!
//! See `docs/caroml/intro.md` for the language reference and
//! `~/.claude/plans/plan-a-caro-dsl-encapsulated-emerson.md` for the
//! design plan that drives this module.
//!
//! # Layout (v0.1)
//!
//! - [`ast`] — parsed `.caro` file representation
//! - [`parser`] — line-keyword tokenizer; no grammar lib needed
//! - [`lock`] — TOML serde for the lock format (schema_version 2)
//!
//! Subsequent PRs add: discovery, runbook writer, validators, interpreter,
//! runner, regen evaluator, history, variants, voice, scaffold, skill,
//! carofile, cve_feed, platform.

pub mod adopt;
pub mod ast;
pub mod carofile;
pub mod discovery;
pub mod history;
pub mod interpreter;
pub mod lock;
pub mod parser;
pub mod platform;
pub mod regen_evaluator;
pub mod runbook;
pub mod runner;
pub mod validators;
pub mod variants;

pub use ast::{Param, ParseError, ParseErrorKind, PlatformPragma, Step, Task};
pub use lock::{Lock, Step as LockStep, Variant};
pub use parser::parse;

use std::path::Path;

/// Read a `.caro` file and parse it, returning the [`Task`] on success.
///
/// This is the high-level convenience used by `caro check`.
pub fn check_file(path: &Path) -> Result<Task, CheckError> {
    let src = std::fs::read_to_string(path).map_err(CheckError::Io)?;
    parser::parse_with_path(&src, Some(path.to_path_buf())).map_err(CheckError::Parse)
}

/// Error from [`check_file`] — either I/O or parsing.
#[derive(Debug)]
pub enum CheckError {
    Io(std::io::Error),
    Parse(ParseError),
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Parse(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
        }
    }
}
