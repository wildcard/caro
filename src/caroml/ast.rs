//! Abstract Syntax Tree for parsed `.caro` files.
//!
//! A `.caro` file is line-oriented; each non-blank, non-`REM` line begins
//! with one of eight keywords. The parser produces a [`Task`] containing
//! exactly one title, optional `WHY`, zero-or-more pragmas, and one-or-more
//! [`Step`]s (the `DO` lines that drive generation).
//!
//! # Example
//! ```text
//! TASK Clean up old log files
//! WHY  Free disk space, runs weekly via cron
//!
//! NEED sudo
//! ON   macos PREFER bsd-tools
//! LET  path = /var/log
//!
//! NOTE prefer single-pass find
//! DO   find log files in {path}
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A fully parsed CaroML task — one `.caro` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Source path on disk, if loaded from a file.
    pub source_path: Option<PathBuf>,

    /// Required: title from the single `TASK` line. Always non-empty.
    pub title: String,

    /// Optional: motivation from a single `WHY` line.
    pub why: Option<String>,

    /// Zero or more `NEED <thing>` declarations (sudo, network, etc.).
    pub needs: Vec<String>,

    /// Zero or more `ON <platform> [PREFER ...] [AVOID ...]` pragmas.
    pub platform_pragmas: Vec<PlatformPragma>,

    /// Zero or more `LET <name> = <value>` parameters, in source order.
    /// Used for `{name}` substitution in `DO` line intents.
    pub params: Vec<Param>,

    /// One or more `DO <intent>` steps. The unit of generation.
    pub steps: Vec<Step>,
}

/// `ON <platform> [PREFER <a, b, ...>] [AVOID <c, d, ...>]`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformPragma {
    /// Lowercase platform identifier: "macos", "linux", "windows", "posix".
    pub platform: String,
    /// Things to prefer on this platform (e.g. `bsd-tools`).
    pub prefer: Vec<String>,
    /// Things to avoid on this platform (e.g. `systemd-cat`).
    pub avoid: Vec<String>,
}

/// `LET <name> = <value>` — an authoring-time parameter.
///
/// Substituted into `DO` lines via `{name}` markers at parse time.
/// Not exposed to the generated shell script as a variable; the LLM is
/// free to materialize it as a shell variable or inline literal as it sees fit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub value: String,
}

/// A single `DO <intent>` step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// 1-based source line number in the original `.caro` file.
    pub line: usize,

    /// Intent text after `{name}` substitution (the form fed to the LLM).
    pub intent: String,

    /// Intent text as written by the human, with `{name}` markers intact.
    /// Preserved so re-rendering and round-trip diffs are faithful.
    pub raw_intent: String,

    /// Any `NOTE` lines that immediately preceded this `DO`, in source order.
    /// These are passed to the LLM as additional guidance for this step.
    pub notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Parser errors
// ---------------------------------------------------------------------------

/// A parse error. Always carries the source line number for editor jump-to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line number where the error was detected.
    pub line: usize,
    /// Optional 1-based column for finer pointing.
    pub col: Option<usize>,
    /// What went wrong.
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(line: usize, kind: ParseErrorKind) -> Self {
        Self {
            line,
            col: None,
            kind,
        }
    }

    pub fn with_col(mut self, col: usize) -> Self {
        self.col = Some(col);
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.col {
            Some(c) => write!(f, "line {}:{}: {}", self.line, c, self.kind),
            None => write!(f, "line {}: {}", self.line, self.kind),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// First non-comment line was not `TASK <title>`.
    MissingTaskHeader,
    /// More than one `TASK` line in the file.
    DuplicateTask,
    /// More than one `WHY` line in the file.
    DuplicateWhy,
    /// A line started with an unknown keyword.
    UnknownKeyword(String),
    /// `LET` line could not be parsed as `LET name = value`.
    MalformedLet,
    /// `ON` line could not be parsed as `ON <platform> [PREFER ...] [AVOID ...]`.
    MalformedOn,
    /// A `{name}` interpolation in a `DO` line was never closed.
    UnclosedInterpolation { line: usize },
    /// `{}` with no name between the braces.
    EmptyInterpolation,
    /// A `{name}` referenced a name not defined by any prior `LET`.
    UndefinedParam(String),
    /// A `TASK` line had no title.
    EmptyTaskTitle,
    /// The file parsed cleanly but had zero `DO` steps.
    NoSteps,
    // ---- Carofile-specific variants (used by `caroml::carofile`) ----
    /// `USE` line could not be parsed as `USE <target> AS <alias>`.
    MalformedUse,
    /// `JOB` had no name.
    EmptyJobName,
    /// `RUN` appeared outside any `JOB` body.
    RunOutsideJob,
    /// `RUN <alias>` referenced an alias not declared by any prior `USE`.
    UndefinedAlias(String),
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTaskHeader => {
                write!(f, "expected `TASK <title>` as the first non-comment line")
            }
            Self::DuplicateTask => write!(f, "more than one `TASK` line"),
            Self::DuplicateWhy => write!(f, "more than one `WHY` line"),
            Self::UnknownKeyword(kw) => write!(f, "unknown keyword `{}`", kw),
            Self::MalformedLet => {
                write!(f, "expected `LET <name> = <value>`")
            }
            Self::MalformedOn => write!(
                f,
                "expected `ON <platform> [PREFER <a,b,...>] [AVOID <c,d,...>]`"
            ),
            Self::UnclosedInterpolation { line } => {
                write!(f, "unclosed `{{...}}` interpolation in DO on line {}", line)
            }
            Self::EmptyInterpolation => {
                write!(
                    f,
                    "empty `{{}}` in DO; either name a parameter or escape with `{{{{`"
                )
            }
            Self::UndefinedParam(name) => write!(
                f,
                "DO references `{{{}}}` but no `LET {} = ...` was declared",
                name, name
            ),
            Self::EmptyTaskTitle => write!(f, "TASK line has no title"),
            Self::NoSteps => write!(f, "task has no `DO` steps"),
            Self::MalformedUse => {
                write!(f, "expected `USE <target> AS <alias>`")
            }
            Self::EmptyJobName => write!(f, "JOB has no name"),
            Self::RunOutsideJob => {
                write!(f, "RUN appeared outside a JOB body")
            }
            Self::UndefinedAlias(name) => write!(
                f,
                "RUN references `{}` but no `USE ... AS {}` was declared",
                name, name
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests for the AST itself (constructors, Display, basic invariants)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_display_with_col() {
        let err = ParseError::new(7, ParseErrorKind::MalformedLet).with_col(12);
        assert_eq!(err.to_string(), "line 7:12: expected `LET <name> = <value>`");
    }

    #[test]
    fn parse_error_display_without_col() {
        let err = ParseError::new(3, ParseErrorKind::DuplicateTask);
        assert_eq!(err.to_string(), "line 3: more than one `TASK` line");
    }

    #[test]
    fn unknown_keyword_message() {
        let err = ParseError::new(1, ParseErrorKind::UnknownKeyword("FOO".into()));
        assert_eq!(err.to_string(), "line 1: unknown keyword `FOO`");
    }

    #[test]
    fn task_struct_round_trip_via_serde_json() {
        let task = Task {
            source_path: None,
            title: "Demo".into(),
            why: Some("for tests".into()),
            needs: vec!["sudo".into()],
            platform_pragmas: vec![PlatformPragma {
                platform: "macos".into(),
                prefer: vec!["bsd-tools".into()],
                avoid: vec![],
            }],
            params: vec![Param {
                name: "path".into(),
                value: "/tmp".into(),
            }],
            steps: vec![Step {
                line: 8,
                intent: "list /tmp".into(),
                raw_intent: "list {path}".into(),
                notes: vec!["be portable".into()],
            }],
        };
        let json = serde_json::to_string(&task).unwrap();
        let back: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task, back);
    }
}
