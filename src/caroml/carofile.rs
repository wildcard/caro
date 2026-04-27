//! Carofile parser — top-level project orchestration file.
//!
//! A `Carofile` (or `Carofile.caro`) sits at the project root and indexes
//! native CaroML tasks alongside external runbook commands (Make targets,
//! `package.json` scripts, ad-hoc shell scripts), then composes them into
//! higher-level **jobs** that `caro do <job>` runs.
//!
//! ## Grammar
//!
//! Reuses the line-keyword vocabulary of `.caro` files plus three new keywords:
//!
//! ```text
//! TASK <project orchestration title>      # required, exactly one
//! WHY  <reason>                            # optional, at most one
//!
//! USE <target> AS <alias>                  # zero or more
//! USE "<external command>" AS <alias>     # quoted strings → external
//! USE <path-to-.caro> AS <alias>           # paths → native task
//!
//! JOB <name>                                # opens a job context
//!   RUN <alias>                             # 1+ run lines, refer to USE aliases
//!   RUN <alias>
//!
//! REM <comment>                             # ignored, may appear anywhere
//! ```
//!
//! Job-body termination: a JOB body is closed by the next top-level keyword
//! (TASK / WHY / USE / JOB) or by EOF. Blank lines and REM comments inside a
//! body don't close it. Indentation is purely cosmetic — ignored.
//!
//! ## v0.1 limitations
//!
//! - No JOB-level NEED / ON / LET (parsed-but-not-enforced is also out of scope
//!   for the parser; v0.2 plans add it).
//! - JOBs run sequentially only (no parallel composition syntax).
//! - `USE … FROM <path>` clause from the design plan is not yet implemented;
//!   reserved for v0.2.

use crate::caroml::ast::{ParseError, ParseErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// A parsed Carofile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carofile {
    pub source_path: Option<PathBuf>,
    pub title: String,
    pub why: Option<String>,
    pub uses: Vec<UseDecl>,
    pub jobs: Vec<Job>,
}

/// `USE <target> AS <alias>` — registers a callable name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseDecl {
    pub line: usize,
    pub alias: String,
    pub target: UseTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseTarget {
    /// Native CaroML task — path to a `.caro` file.
    NativeTask(PathBuf),
    /// External shell command (originally quoted in the Carofile).
    ExternalCommand(String),
}

/// `JOB <name>` with its body of `RUN <alias>` lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub line: usize,
    pub name: String,
    /// Alias names referenced by `RUN` lines, in source order.
    pub runs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a Carofile from a string.
pub fn parse(src: &str) -> Result<Carofile, ParseError> {
    parse_with_path(src, None)
}

/// Parse a Carofile with an associated source path.
pub fn parse_with_path(src: &str, source_path: Option<PathBuf>) -> Result<Carofile, ParseError> {
    let mut state = State::new(source_path);
    for (idx, raw_line) in src.lines().enumerate() {
        state.line_no = idx + 1;
        let line = raw_line.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        state.last_line = state.line_no;

        let (keyword, rest) = split_keyword(trimmed);
        match keyword {
            "REM" => continue,
            "TASK" => {
                state.close_active_job();
                state.handle_task(rest)?;
            }
            "WHY" => {
                state.close_active_job();
                state.handle_why(rest)?;
            }
            "USE" => {
                state.close_active_job();
                state.handle_use(rest)?;
            }
            "JOB" => {
                state.close_active_job();
                state.handle_job(rest)?;
            }
            "RUN" => state.handle_run(rest)?,
            other => {
                return Err(ParseError::new(
                    state.line_no,
                    ParseErrorKind::UnknownKeyword(other.to_string()),
                ));
            }
        }
    }
    state.finish()
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

struct State {
    line_no: usize,
    last_line: usize,
    source_path: Option<PathBuf>,
    title: Option<String>,
    why: Option<String>,
    uses: Vec<UseDecl>,
    jobs: Vec<Job>,
    /// Currently being assembled, if any.
    active_job: Option<Job>,
}

impl State {
    fn new(source_path: Option<PathBuf>) -> Self {
        Self {
            line_no: 0,
            last_line: 0,
            source_path,
            title: None,
            why: None,
            uses: Vec::new(),
            jobs: Vec::new(),
            active_job: None,
        }
    }

    fn handle_task(&mut self, rest: &str) -> Result<(), ParseError> {
        if self.title.is_some() {
            return Err(ParseError::new(self.line_no, ParseErrorKind::DuplicateTask));
        }
        let title = rest.trim();
        if title.is_empty() {
            return Err(ParseError::new(
                self.line_no,
                ParseErrorKind::EmptyTaskTitle,
            ));
        }
        self.title = Some(title.to_string());
        Ok(())
    }

    fn handle_why(&mut self, rest: &str) -> Result<(), ParseError> {
        self.require_title()?;
        if self.why.is_some() {
            return Err(ParseError::new(self.line_no, ParseErrorKind::DuplicateWhy));
        }
        self.why = Some(rest.trim().to_string());
        Ok(())
    }

    fn handle_use(&mut self, rest: &str) -> Result<(), ParseError> {
        self.require_title()?;
        let parsed = parse_use_clause(rest).ok_or_else(|| {
            ParseError::new(self.line_no, ParseErrorKind::MalformedUse)
        })?;
        self.uses.push(UseDecl {
            line: self.line_no,
            alias: parsed.0,
            target: parsed.1,
        });
        Ok(())
    }

    fn handle_job(&mut self, rest: &str) -> Result<(), ParseError> {
        self.require_title()?;
        let name = rest.trim();
        if name.is_empty() {
            return Err(ParseError::new(self.line_no, ParseErrorKind::EmptyJobName));
        }
        self.active_job = Some(Job {
            line: self.line_no,
            name: name.to_string(),
            runs: Vec::new(),
        });
        Ok(())
    }

    fn handle_run(&mut self, rest: &str) -> Result<(), ParseError> {
        let alias = rest.trim();
        match self.active_job.as_mut() {
            Some(job) => {
                if !alias.is_empty() {
                    job.runs.push(alias.to_string());
                }
                Ok(())
            }
            None => Err(ParseError::new(
                self.line_no,
                ParseErrorKind::RunOutsideJob,
            )),
        }
    }

    fn close_active_job(&mut self) {
        if let Some(job) = self.active_job.take() {
            self.jobs.push(job);
        }
    }

    fn require_title(&self) -> Result<(), ParseError> {
        if self.title.is_none() {
            Err(ParseError::new(
                self.line_no,
                ParseErrorKind::MissingTaskHeader,
            ))
        } else {
            Ok(())
        }
    }

    fn finish(mut self) -> Result<Carofile, ParseError> {
        self.close_active_job();

        let eof_line = self.last_line.max(1);
        let title = self
            .title
            .ok_or_else(|| ParseError::new(eof_line, ParseErrorKind::MissingTaskHeader))?;

        // Validate that every RUN references a declared USE alias.
        let known_aliases: HashSet<&str> =
            self.uses.iter().map(|u| u.alias.as_str()).collect();
        for job in &self.jobs {
            for alias in &job.runs {
                if !known_aliases.contains(alias.as_str()) {
                    return Err(ParseError::new(
                        job.line,
                        ParseErrorKind::UndefinedAlias(alias.clone()),
                    ));
                }
            }
        }

        Ok(Carofile {
            source_path: self.source_path,
            title,
            why: self.why,
            uses: self.uses,
            jobs: self.jobs,
        })
    }
}

// ---------------------------------------------------------------------------
// Sub-parsers
// ---------------------------------------------------------------------------

fn split_keyword(line: &str) -> (&str, &str) {
    match line.split_once(char::is_whitespace) {
        Some((kw, rest)) => (kw, rest),
        None => (line, ""),
    }
}

/// Parse the body of a `USE` line: `<target> AS <alias>`.
///
/// `<target>` is either `"a quoted command"` (external) or a path-like literal
/// (native task). The `AS <alias>` clause is required.
fn parse_use_clause(rest: &str) -> Option<(String, UseTarget)> {
    let trimmed = rest.trim_start();
    let (target, after_target) = take_target(trimmed)?;
    let after = after_target.trim_start();
    let after_lower = after.to_uppercase();
    let alias_part = after_lower.strip_prefix("AS")?;
    if !alias_part.starts_with(char::is_whitespace) {
        return None;
    }
    let alias = after[2..].trim();
    if alias.is_empty() {
        return None;
    }
    // Alias must be a single token (no spaces).
    if alias.contains(char::is_whitespace) {
        return None;
    }
    Some((alias.to_string(), target))
}

/// Pull the `<target>` portion off the front: a quoted string or a single token.
/// Returns `(target, remaining)` on success.
fn take_target(s: &str) -> Option<(UseTarget, &str)> {
    if let Some(rest) = s.strip_prefix('"') {
        let close = rest.find('"')?;
        let cmd = &rest[..close];
        if cmd.is_empty() {
            return None;
        }
        Some((UseTarget::ExternalCommand(cmd.to_string()), &rest[close + 1..]))
    } else {
        let end = s
            .find(char::is_whitespace)
            .unwrap_or(s.len());
        let token = &s[..end];
        if token.is_empty() {
            return None;
        }
        Some((UseTarget::NativeTask(PathBuf::from(token)), &s[end..]))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn must_parse(src: &str) -> Carofile {
        parse(src).expect("expected successful Carofile parse")
    }

    #[test]
    fn parses_minimal_carofile() {
        let src = "TASK Project orchestration\n";
        let cf = must_parse(src);
        assert_eq!(cf.title, "Project orchestration");
        assert!(cf.why.is_none());
        assert!(cf.uses.is_empty());
        assert!(cf.jobs.is_empty());
    }

    #[test]
    fn parses_use_with_external_quoted_command() {
        let src = "TASK demo\nUSE \"npm test\" AS test\n";
        let cf = must_parse(src);
        assert_eq!(cf.uses.len(), 1);
        assert_eq!(cf.uses[0].alias, "test");
        assert!(matches!(cf.uses[0].target, UseTarget::ExternalCommand(ref s) if s == "npm test"));
    }

    #[test]
    fn parses_use_with_native_task_path() {
        let src = "TASK demo\nUSE tasks/cleanup-logs.caro AS cleanup-logs\n";
        let cf = must_parse(src);
        let use_decl = &cf.uses[0];
        assert_eq!(use_decl.alias, "cleanup-logs");
        match &use_decl.target {
            UseTarget::NativeTask(p) => {
                assert_eq!(p.to_string_lossy(), "tasks/cleanup-logs.caro")
            }
            _ => panic!("expected native task target"),
        }
    }

    #[test]
    fn parses_job_with_run_lines() {
        let src = "\
TASK demo
USE \"make build\" AS build
USE \"npm test\" AS test
JOB ci
  RUN build
  RUN test
";
        let cf = must_parse(src);
        assert_eq!(cf.jobs.len(), 1);
        assert_eq!(cf.jobs[0].name, "ci");
        assert_eq!(cf.jobs[0].runs, vec!["build".to_string(), "test".to_string()]);
    }

    #[test]
    fn job_terminates_at_next_top_level_keyword() {
        let src = "\
TASK demo
USE \"make build\" AS build
USE \"npm test\" AS test
USE \"cargo clippy\" AS lint
JOB ci
  RUN build
  RUN test
JOB nightly
  RUN lint
";
        let cf = must_parse(src);
        assert_eq!(cf.jobs.len(), 2);
        assert_eq!(cf.jobs[0].name, "ci");
        assert_eq!(cf.jobs[0].runs, vec!["build".to_string(), "test".to_string()]);
        assert_eq!(cf.jobs[1].name, "nightly");
        assert_eq!(cf.jobs[1].runs, vec!["lint".to_string()]);
    }

    #[test]
    fn rem_inside_job_body_is_ignored() {
        let src = "\
TASK demo
USE \"x\" AS x
JOB j
  REM this is a comment in the body
  RUN x
";
        let cf = must_parse(src);
        assert_eq!(cf.jobs[0].runs, vec!["x".to_string()]);
    }

    #[test]
    fn run_outside_job_is_error() {
        let src = "TASK demo\nUSE \"x\" AS x\nRUN x\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::RunOutsideJob));
        assert_eq!(err.line, 3);
    }

    #[test]
    fn run_referencing_undeclared_alias_is_error() {
        let src = "TASK demo\nUSE \"x\" AS x\nJOB j\n  RUN nope\n";
        let err = parse(src).unwrap_err();
        match err.kind {
            ParseErrorKind::UndefinedAlias(ref n) if n == "nope" => {}
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn malformed_use_without_alias_is_error() {
        let src = "TASK demo\nUSE \"npm test\"\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::MalformedUse));
    }

    #[test]
    fn malformed_use_alias_with_spaces_is_error() {
        // Alias must be a single token.
        let src = "TASK demo\nUSE \"npm test\" AS my alias\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::MalformedUse));
    }

    #[test]
    fn empty_job_name_is_error() {
        let src = "TASK demo\nJOB\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::EmptyJobName));
    }

    #[test]
    fn missing_task_header_is_error() {
        let src = "USE \"npm test\" AS test\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::MissingTaskHeader));
    }

    #[test]
    fn full_carofile_round_trip() {
        let src = "\
REM Carofile demo
TASK Project orchestration
WHY  Single front door for repeatable tasks; augments Makefile.

USE   tasks/cleanup-logs.caro       AS cleanup-logs
USE   \"npm test\"                    AS test
USE   \"make build\"                  AS build
USE   \"cargo clippy --workspace\"    AS lint

JOB ci
  RUN lint
  RUN test
  RUN build

JOB nightly
  RUN cleanup-logs
";
        let cf = must_parse(src);
        assert_eq!(cf.title, "Project orchestration");
        assert_eq!(cf.uses.len(), 4);
        assert_eq!(cf.jobs.len(), 2);
        assert_eq!(
            cf.jobs[0].runs,
            vec![
                "lint".to_string(),
                "test".to_string(),
                "build".to_string()
            ]
        );
        assert_eq!(cf.jobs[1].runs, vec!["cleanup-logs".to_string()]);

        // Round-trip through serde_json to confirm Serialize/Deserialize.
        let json = serde_json::to_string(&cf).unwrap();
        let back: Carofile = serde_json::from_str(&json).unwrap();
        assert_eq!(cf, back);
    }
}
