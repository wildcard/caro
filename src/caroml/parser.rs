//! Line-keyword tokenizer and parser for `.caro` files.
//!
//! The grammar is trivial: each non-blank line begins with one of eight
//! keywords (TASK, WHY, NEED, ON, LET, DO, NOTE, REM). The first non-`REM`
//! token determines the line kind; the rest of the line is the payload
//! interpreted per-keyword.
//!
//! No grammar library is used — a hand-rolled tokenizer is enough.
//!
//! # Error strategy
//!
//! v0.1 is **fail-fast**: parsing returns `Err(first_error)` on the first
//! problem encountered. This keeps the implementation simple and gives
//! editor jump-to-line behaviour. A future version may switch to an
//! error-recovery (collect-all) mode for `caro check`; the public API
//! returns a single `ParseError` today but the type is shaped to allow
//! a `Vec<ParseError>` variant later without source-breaking changes.

use crate::caroml::ast::{ParseError, ParseErrorKind, Param, PlatformPragma, Step, Task};
use std::collections::HashSet;
use std::path::PathBuf;

/// Parse a `.caro` source string into a [`Task`].
///
/// Returns the first parse error encountered (fail-fast).
pub fn parse(src: &str) -> Result<Task, ParseError> {
    parse_with_path(src, None)
}

/// Parse with an associated source path (for error messages and the `Task.source_path` field).
pub fn parse_with_path(src: &str, source_path: Option<PathBuf>) -> Result<Task, ParseError> {
    let mut state = ParseState::new(source_path);
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
            "TASK" => state.handle_task(rest)?,
            "WHY" => state.handle_why(rest)?,
            "NEED" => state.handle_need(rest)?,
            "ON" => state.handle_on(rest)?,
            "LET" => state.handle_let(rest)?,
            "NOTE" => state.handle_note(rest),
            "DO" => state.handle_do(rest)?,
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
// Internal state
// ---------------------------------------------------------------------------

struct ParseState {
    line_no: usize,
    /// Last non-blank line seen — used as the location for end-of-file errors.
    last_line: usize,
    source_path: Option<PathBuf>,
    title: Option<String>,
    why: Option<String>,
    needs: Vec<String>,
    pragmas: Vec<PlatformPragma>,
    params: Vec<Param>,
    pending_notes: Vec<String>,
    steps: Vec<Step>,
}

impl ParseState {
    fn new(source_path: Option<PathBuf>) -> Self {
        Self {
            line_no: 0,
            last_line: 0,
            source_path,
            title: None,
            why: None,
            needs: Vec::new(),
            pragmas: Vec::new(),
            params: Vec::new(),
            pending_notes: Vec::new(),
            steps: Vec::new(),
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
        self.require_task()?;
        if self.why.is_some() {
            return Err(ParseError::new(self.line_no, ParseErrorKind::DuplicateWhy));
        }
        self.why = Some(rest.trim().to_string());
        Ok(())
    }

    fn handle_need(&mut self, rest: &str) -> Result<(), ParseError> {
        self.require_task()?;
        let item = rest.trim();
        if !item.is_empty() {
            self.needs.push(item.to_string());
        }
        Ok(())
    }

    fn handle_on(&mut self, rest: &str) -> Result<(), ParseError> {
        self.require_task()?;
        let parsed = parse_on_clause(rest).ok_or_else(|| {
            ParseError::new(self.line_no, ParseErrorKind::MalformedOn)
        })?;
        self.pragmas.push(parsed);
        Ok(())
    }

    fn handle_let(&mut self, rest: &str) -> Result<(), ParseError> {
        self.require_task()?;
        let (name, value) = parse_let_clause(rest)
            .ok_or_else(|| ParseError::new(self.line_no, ParseErrorKind::MalformedLet))?;
        self.params.push(Param { name, value });
        Ok(())
    }

    fn handle_note(&mut self, rest: &str) {
        let note = rest.trim();
        if !note.is_empty() {
            self.pending_notes.push(note.to_string());
        }
    }

    fn handle_do(&mut self, rest: &str) -> Result<(), ParseError> {
        self.require_task()?;
        let raw_intent = rest.trim().to_string();
        let intent = substitute_params(&raw_intent, &self.params, self.line_no)?;
        let notes = std::mem::take(&mut self.pending_notes);
        self.steps.push(Step {
            line: self.line_no,
            intent,
            raw_intent,
            notes,
        });
        Ok(())
    }

    fn require_task(&self) -> Result<(), ParseError> {
        if self.title.is_none() {
            Err(ParseError::new(
                self.line_no,
                ParseErrorKind::MissingTaskHeader,
            ))
        } else {
            Ok(())
        }
    }

    fn finish(self) -> Result<Task, ParseError> {
        // End-of-file errors point at the last non-blank line we saw, falling
        // back to line 1 for entirely empty / whitespace-only files. Never 0.
        let eof_line = self.last_line.max(1);
        let title = self
            .title
            .ok_or_else(|| ParseError::new(eof_line, ParseErrorKind::MissingTaskHeader))?;
        if self.steps.is_empty() {
            return Err(ParseError::new(eof_line, ParseErrorKind::NoSteps));
        }
        Ok(Task {
            source_path: self.source_path,
            title,
            why: self.why,
            needs: self.needs,
            platform_pragmas: self.pragmas,
            params: self.params,
            steps: self.steps,
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

fn parse_let_clause(rest: &str) -> Option<(String, String)> {
    let (lhs, rhs) = rest.split_once('=')?;
    let name = lhs.trim();
    let value = rhs.trim();
    if name.is_empty() || !is_valid_identifier(name) {
        return None;
    }
    Some((name.to_string(), value.to_string()))
}

fn parse_on_clause(rest: &str) -> Option<PlatformPragma> {
    let mut tokens = rest.split_whitespace();
    let platform = tokens.next()?.to_lowercase();
    if !is_valid_platform(&platform) {
        return None;
    }
    let mut prefer = Vec::new();
    let mut avoid = Vec::new();
    let mut current: Option<&mut Vec<String>> = None;
    for tok in tokens {
        match tok.to_uppercase().as_str() {
            "PREFER" => current = Some(&mut prefer),
            "AVOID" => current = Some(&mut avoid),
            _ => {
                if let Some(target) = current.as_deref_mut() {
                    for item in tok.split(',') {
                        let trimmed = item.trim();
                        if !trimmed.is_empty() {
                            target.push(trimmed.to_string());
                        }
                    }
                } else {
                    return None;
                }
            }
        }
    }
    Some(PlatformPragma {
        platform,
        prefer,
        avoid,
    })
}

/// Substitute `{name}` placeholders in `raw` with `params` values.
///
/// Escape rules:
/// - `{{` is a literal `{` (lets `awk '{{print $1}}'` survive parsing — see issue
///   linked from PR review). The matching `}}` is also collapsed to `}` for symmetry.
/// - `{name}` references a `LET name = value`; substitutes the value.
/// - `{}` is an error (`EmptyInterpolation`).
/// - `{name` without closing `}` is `UnclosedInterpolation`.
///
/// UTF-8 safe: walks `raw` by char with byte-tracking, never indexes by raw byte
/// for the literal-copy branch.
fn substitute_params(
    raw: &str,
    params: &[Param],
    line_no: usize,
) -> Result<String, ParseError> {
    let mut out = String::with_capacity(raw.len());
    let known: HashSet<&str> = params.iter().map(|p| p.name.as_str()).collect();

    let mut i = 0usize;
    let bytes = raw.as_bytes();

    while i < bytes.len() {
        // Literal `{` escape: `{{` → `{`
        if bytes[i] == b'{' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            out.push('{');
            i += 2;
            continue;
        }
        // Literal `}` escape: `}}` → `}` (symmetric)
        if bytes[i] == b'}' && i + 1 < bytes.len() && bytes[i + 1] == b'}' {
            out.push('}');
            i += 2;
            continue;
        }

        if bytes[i] == b'{' {
            // {name} interpolation
            let close = match bytes[i + 1..].iter().position(|&b| b == b'}') {
                Some(rel) => i + 1 + rel,
                None => {
                    return Err(ParseError::new(
                        line_no,
                        ParseErrorKind::UnclosedInterpolation { line: line_no },
                    ));
                }
            };
            let name = &raw[i + 1..close];
            if name.is_empty() {
                return Err(ParseError::new(line_no, ParseErrorKind::EmptyInterpolation));
            }
            if !known.contains(name) {
                return Err(ParseError::new(
                    line_no,
                    ParseErrorKind::UndefinedParam(name.to_string()),
                ));
            }
            let value = params
                .iter()
                .find(|p| p.name == name)
                .map(|p| p.value.as_str())
                .unwrap_or("");
            out.push_str(value);
            i = close + 1;
            continue;
        }

        // Literal copy — UTF-8 safe: read one char from raw[i..] and advance by its byte length.
        let c = raw[i..].chars().next().unwrap();
        let len = c.len_utf8();
        out.push(c);
        i += len;
    }
    Ok(out)
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn is_valid_platform(s: &str) -> bool {
    matches!(s, "macos" | "linux" | "windows" | "posix")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn must_parse(src: &str) -> Task {
        parse(src).expect("expected successful parse")
    }

    #[test]
    fn parses_minimal_task() {
        let src = "TASK Hello\nDO say hi\n";
        let task = must_parse(src);
        assert_eq!(task.title, "Hello");
        assert_eq!(task.steps.len(), 1);
        assert_eq!(task.steps[0].intent, "say hi");
        assert_eq!(task.steps[0].line, 2);
    }

    #[test]
    fn ignores_blank_lines_and_rem() {
        let src = "\nREM hello\nTASK Hello\n\nREM another comment\nDO say hi\n";
        let task = must_parse(src);
        assert_eq!(task.title, "Hello");
        assert_eq!(task.steps.len(), 1);
    }

    #[test]
    fn task_with_why_and_needs() {
        let src = "TASK Demo\nWHY because\nNEED sudo\nNEED jq\nDO go\n";
        let task = must_parse(src);
        assert_eq!(task.why, Some("because".into()));
        assert_eq!(task.needs, vec!["sudo".to_string(), "jq".into()]);
    }

    #[test]
    fn parses_on_with_prefer_and_avoid() {
        let src =
            "TASK Demo\nON macos PREFER bsd-tools AVOID gnu-tools, systemd\nDO go\n";
        let task = must_parse(src);
        let p = &task.platform_pragmas[0];
        assert_eq!(p.platform, "macos");
        assert_eq!(p.prefer, vec!["bsd-tools".to_string()]);
        assert_eq!(
            p.avoid,
            vec!["gnu-tools".to_string(), "systemd".to_string()]
        );
    }

    #[test]
    fn rejects_on_unknown_platform() {
        let src = "TASK Demo\nON solaris PREFER bsd-tools\nDO go\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::MalformedOn));
        assert_eq!(err.line, 2);
    }

    #[test]
    fn substitutes_let_into_intent() {
        let src = "TASK Demo\nLET path = /tmp\nDO list {path}\n";
        let task = must_parse(src);
        assert_eq!(task.steps[0].intent, "list /tmp");
        assert_eq!(task.steps[0].raw_intent, "list {path}");
    }

    #[test]
    fn rejects_undefined_param() {
        let src = "TASK Demo\nDO list {nope}\n";
        let err = parse(src).unwrap_err();
        match err.kind {
            ParseErrorKind::UndefinedParam(ref n) if n == "nope" => {}
            other => panic!("unexpected error: {:?}", other),
        }
        assert_eq!(err.line, 2);
    }

    #[test]
    fn rejects_unclosed_interpolation() {
        let src = "TASK Demo\nLET path = /tmp\nDO list {path\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(
            err.kind,
            ParseErrorKind::UnclosedInterpolation { .. }
        ));
        assert_eq!(err.line, 3);
    }

    #[test]
    fn note_attaches_to_next_do() {
        let src = "TASK Demo\nNOTE be portable\nNOTE prefer xargs\nDO list /tmp\nDO clean up\n";
        let task = must_parse(src);
        assert_eq!(task.steps.len(), 2);
        assert_eq!(
            task.steps[0].notes,
            vec!["be portable".to_string(), "prefer xargs".to_string()]
        );
        assert!(task.steps[1].notes.is_empty());
    }

    #[test]
    fn duplicate_task_is_error() {
        let src = "TASK Demo\nTASK Other\nDO go\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::DuplicateTask));
        assert_eq!(err.line, 2);
    }

    #[test]
    fn duplicate_why_is_error() {
        let src = "TASK Demo\nWHY because\nWHY also because\nDO go\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::DuplicateWhy));
        assert_eq!(err.line, 3);
    }

    #[test]
    fn missing_task_header_is_error() {
        let src = "DO go\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::MissingTaskHeader));
    }

    #[test]
    fn empty_task_title_is_error() {
        let src = "TASK   \nDO go\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::EmptyTaskTitle));
    }

    #[test]
    fn no_steps_is_error() {
        let src = "TASK Demo\nWHY because\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::NoSteps));
    }

    #[test]
    fn unknown_keyword_is_error() {
        let src = "TASK Demo\nFOO bar\nDO go\n";
        let err = parse(src).unwrap_err();
        match err.kind {
            ParseErrorKind::UnknownKeyword(ref kw) if kw == "FOO" => {}
            other => panic!("unexpected error: {:?}", other),
        }
        assert_eq!(err.line, 2);
    }

    #[test]
    fn utf8_in_intent_round_trips_intact() {
        // Regression: prior implementation truncated non-ASCII codepoints via
        // `bytes[i] as char`. café / 日本語 / 🌱 must survive verbatim.
        let src = "TASK Demo\nDO open the café\nDO greet 日本語 🌱\n";
        let task = must_parse(src);
        assert_eq!(task.steps[0].intent, "open the café");
        assert_eq!(task.steps[1].intent, "greet 日本語 🌱");
    }

    #[test]
    fn double_brace_escape_yields_literal_brace() {
        // Lets shell snippets like `awk '{{print $1}}'` survive parsing.
        let src = "TASK Demo\nDO awk '{{print $1}}' on the file\n";
        let task = must_parse(src);
        assert_eq!(task.steps[0].intent, "awk '{print $1}' on the file");
    }

    #[test]
    fn empty_interpolation_is_explicit_error() {
        let src = "TASK Demo\nDO list {}\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::EmptyInterpolation));
        assert_eq!(err.line, 2);
    }

    #[test]
    fn missing_task_uses_last_seen_line_not_zero() {
        let src = "REM intro\nWHY because\nNEED sudo\n";
        let err = parse(src).unwrap_err();
        // First non-comment, non-blank line is `WHY` on line 2 — that's where
        // the missing `TASK` requirement bites.
        assert!(matches!(err.kind, ParseErrorKind::MissingTaskHeader));
        assert_eq!(err.line, 2);
    }

    #[test]
    fn no_steps_uses_last_seen_line_not_zero() {
        let src = "TASK Demo\nWHY because\nNEED sudo\n";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::NoSteps));
        // `NEED` was the last meaningful line — line 3.
        assert_eq!(err.line, 3);
    }

    #[test]
    fn empty_file_uses_line_one() {
        let src = "";
        let err = parse(src).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::MissingTaskHeader));
        assert_eq!(err.line, 1);
    }

    #[test]
    fn full_cleanup_logs_example() {
        let src = "\
TASK Clean up old log files
WHY  Free disk space and rotate; runs weekly via cron

NEED sudo
ON   macos PREFER bsd-tools
ON   linux PREFER gnu-tools

LET  path = /var/log
LET  days = 30

NOTE prefer single-pass find, avoid spawning a subshell per file
DO   find log files in {path}
DO   filter to those older than {days} days
DO   delete each one, asking confirmation per file
DO   record what was deleted to /tmp/caro-cleanup.log
";
        let task = must_parse(src);
        assert_eq!(task.title, "Clean up old log files");
        assert_eq!(
            task.why.as_deref(),
            Some("Free disk space and rotate; runs weekly via cron")
        );
        assert_eq!(task.needs, vec!["sudo".to_string()]);
        assert_eq!(task.platform_pragmas.len(), 2);
        assert_eq!(task.params.len(), 2);
        assert_eq!(task.steps.len(), 4);
        assert_eq!(
            task.steps[0].intent,
            "find log files in /var/log"
        );
        assert_eq!(
            task.steps[0].raw_intent,
            "find log files in {path}"
        );
        assert_eq!(
            task.steps[0].notes,
            vec!["prefer single-pass find, avoid spawning a subshell per file".to_string()]
        );
        assert_eq!(
            task.steps[1].intent,
            "filter to those older than 30 days"
        );
    }
}
