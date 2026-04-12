//! Shell expansion detection
//!
//! Detects command substitution, variable interpolation, and process substitution
//! in shell commands before the safety pattern matcher runs.
//!
//! # Why This Matters
//!
//! The regex-based safety pattern database in `patterns.rs` can detect known
//! dangerous command forms like `rm -rf /`, but it cannot see *inside* shell
//! expansions. A command like `echo $(rm -rf /)` contains a dangerous operation
//! that the top-level patterns will not match.
//!
//! # OES Inspiration
//!
//! OpenEndpointSecurity validates all event arguments at the kernel boundary
//! before passing them to userspace clients. We apply the same principle:
//! detect shell metacharacters that could alter the meaning of a command
//! *before* the safety pattern matcher runs, and raise the risk level
//! accordingly.
//!
//! # Detected Forms
//!
//! - `$(...)` -- POSIX command substitution (nested-paren aware)
//! - `` `...` `` -- Legacy command substitution
//! - `${...}` -- Parameter expansion with modifiers
//! - `$VAR` -- Simple variable reference
//! - `<(...)`, `>(...)` -- Process substitution (bash/zsh)
//! - `$((...))` -- Arithmetic expansion
//!
//! # Quote Awareness
//!
//! On POSIX shells, single-quoted strings are literal: `echo '$(whoami)'`
//! does not execute `whoami`. Double-quoted strings still perform expansions.
//! This detector is quote-aware and will not flag expansions inside single
//! quotes.

use serde::{Deserialize, Serialize};

use crate::models::ShellType;

/// A detected shell expansion in a command string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellExpansion {
    /// Type of expansion detected
    pub kind: ExpansionKind,
    /// Byte offset in the original command where the expansion starts
    pub start: usize,
    /// Byte offset where the expansion ends (exclusive)
    pub end: usize,
    /// Human-readable description of the expansion
    pub description: String,
}

/// Categories of shell expansions we detect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpansionKind {
    /// `$(command)` -- POSIX command substitution
    CommandSubstitution,
    /// `` `command` `` -- Legacy command substitution
    BacktickSubstitution,
    /// `${var}`, `${var:-default}` -- Parameter expansion
    ParameterExpansion,
    /// `$VAR` -- Simple variable reference
    VariableReference,
    /// `<(command)`, `>(command)` -- Process substitution
    ProcessSubstitution,
    /// `$((expression))` -- Arithmetic expansion
    ArithmeticExpansion,
}

impl ExpansionKind {
    /// Human-readable name for this expansion kind
    pub fn name(&self) -> &'static str {
        match self {
            Self::CommandSubstitution => "command substitution",
            Self::BacktickSubstitution => "backtick command substitution",
            Self::ParameterExpansion => "parameter expansion",
            Self::VariableReference => "variable reference",
            Self::ProcessSubstitution => "process substitution",
            Self::ArithmeticExpansion => "arithmetic expansion",
        }
    }

    /// Whether this kind executes an arbitrary command (high risk) or just
    /// reads a variable value (lower risk)
    pub fn executes_command(&self) -> bool {
        matches!(
            self,
            Self::CommandSubstitution | Self::BacktickSubstitution | Self::ProcessSubstitution
        )
    }
}

/// Detector for shell expansions
///
/// Stateless -- the only configuration is the shell type, which affects
/// quoting semantics. Fish shell, for example, uses `(cmd)` instead of
/// `$(cmd)` for command substitution.
#[derive(Debug, Clone, Copy)]
pub struct ExpansionDetector {
    shell: ShellType,
}

impl ExpansionDetector {
    /// Create a detector for the given shell
    pub fn new(shell: ShellType) -> Self {
        Self { shell }
    }

    /// Scan a command for shell expansions
    ///
    /// Returns all detected expansions in order of occurrence.
    ///
    /// This is quote-aware: expansions inside single-quoted strings on POSIX
    /// shells are considered literal and are not reported. PowerShell and Cmd
    /// use different quoting rules and are not analyzed for expansions here
    /// (they have their own dangerous patterns in `patterns.rs`).
    pub fn detect(&self, command: &str) -> Vec<ShellExpansion> {
        // PowerShell and Cmd have different syntax; skip them for now.
        // Their dangerous forms are covered by the pattern database.
        if matches!(self.shell, ShellType::PowerShell | ShellType::Cmd) {
            return Vec::new();
        }

        let bytes = command.as_bytes();
        let mut results = Vec::new();
        let mut i = 0;
        // Quote state: None = unquoted, Some('\'') = single, Some('"') = double
        let mut in_quote: Option<u8> = None;

        while i < bytes.len() {
            let c = bytes[i];

            // Handle quote state transitions
            if let Some(q) = in_quote {
                if c == q {
                    in_quote = None;
                    i += 1;
                    continue;
                }
                // Inside single quotes, no expansions are performed on POSIX shells
                if q == b'\'' {
                    i += 1;
                    continue;
                }
                // Inside double quotes: command substitution and variable
                // references still happen, so we fall through to detection.
            } else {
                if c == b'\'' || c == b'"' {
                    in_quote = Some(c);
                    i += 1;
                    continue;
                }
            }

            // Handle backslash escaping in unquoted / double-quoted context
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            // `$...` family of expansions
            if c == b'$' && i + 1 < bytes.len() {
                let next = bytes[i + 1];
                // $((expr)) arithmetic expansion (must check before $(cmd))
                if next == b'(' && i + 2 < bytes.len() && bytes[i + 2] == b'(' {
                    if let Some(end) = find_matching_double_paren(bytes, i + 1) {
                        results.push(ShellExpansion {
                            kind: ExpansionKind::ArithmeticExpansion,
                            start: i,
                            end: end + 1,
                            description: format!(
                                "arithmetic expansion: {}",
                                safe_slice(command, i, end + 1)
                            ),
                        });
                        i = end + 1;
                        continue;
                    }
                }
                // $(cmd) command substitution
                if next == b'(' {
                    if let Some(end) = find_matching_paren(bytes, i + 1) {
                        results.push(ShellExpansion {
                            kind: ExpansionKind::CommandSubstitution,
                            start: i,
                            end: end + 1,
                            description: format!(
                                "command substitution: {}",
                                safe_slice(command, i, end + 1)
                            ),
                        });
                        i = end + 1;
                        continue;
                    }
                }
                // ${var} parameter expansion
                if next == b'{' {
                    if let Some(end) = find_matching_brace(bytes, i + 1) {
                        results.push(ShellExpansion {
                            kind: ExpansionKind::ParameterExpansion,
                            start: i,
                            end: end + 1,
                            description: format!(
                                "parameter expansion: {}",
                                safe_slice(command, i, end + 1)
                            ),
                        });
                        i = end + 1;
                        continue;
                    }
                }
                // $VAR simple variable reference
                if next.is_ascii_alphabetic() || next == b'_' {
                    let var_end = scan_identifier(bytes, i + 1);
                    results.push(ShellExpansion {
                        kind: ExpansionKind::VariableReference,
                        start: i,
                        end: var_end,
                        description: format!(
                            "variable reference: {}",
                            safe_slice(command, i, var_end)
                        ),
                    });
                    i = var_end;
                    continue;
                }
            }

            // Backtick substitution: `cmd`
            // Only valid outside single quotes (handled above) and backticks
            // don't nest in a meaningful way in POSIX shells.
            if c == b'`' && in_quote != Some(b'\'') {
                if let Some(end) = find_matching_backtick(bytes, i) {
                    results.push(ShellExpansion {
                        kind: ExpansionKind::BacktickSubstitution,
                        start: i,
                        end: end + 1,
                        description: format!(
                            "backtick substitution: {}",
                            safe_slice(command, i, end + 1)
                        ),
                    });
                    i = end + 1;
                    continue;
                }
            }

            // Process substitution: <(cmd) or >(cmd)
            // Only detected in unquoted context
            if in_quote.is_none()
                && (c == b'<' || c == b'>')
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'('
            {
                if let Some(end) = find_matching_paren(bytes, i + 1) {
                    results.push(ShellExpansion {
                        kind: ExpansionKind::ProcessSubstitution,
                        start: i,
                        end: end + 1,
                        description: format!(
                            "process substitution: {}",
                            safe_slice(command, i, end + 1)
                        ),
                    });
                    i = end + 1;
                    continue;
                }
            }

            i += 1;
        }

        results
    }
}

/// Find the position of the `)` matching the `(` at `open_idx`, accounting
/// for nesting. Returns None if unmatched.
fn find_matching_paren(bytes: &[u8], open_idx: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open_idx], b'(');
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Find the position of `))` matching `((` at `open_idx`. Returns the index
/// of the second `)`. Used for arithmetic expansion `$((...))`.
fn find_matching_double_paren(bytes: &[u8], open_idx: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open_idx], b'(');
    debug_assert_eq!(bytes[open_idx + 1], b'(');
    let mut depth = 0i32;
    let mut i = open_idx;
    // Track when we see ))
    while i + 1 < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' if bytes[i + 1] == b')' && depth == 2 => {
                return Some(i + 1);
            }
            b')' => {
                depth -= 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Find matching `}` for `{` at open_idx
fn find_matching_brace(bytes: &[u8], open_idx: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open_idx], b'{');
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Find matching backtick for backtick at `open_idx`. Does not handle nested
/// backticks (which require `\`` escaping in POSIX).
fn find_matching_backtick(bytes: &[u8], open_idx: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open_idx], b'`');
    let mut i = open_idx + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'`' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

/// Scan an identifier starting at `start` (first char must be valid identifier
/// start). Returns the index one past the end of the identifier.
fn scan_identifier(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphanumeric() || c == b'_' {
            i += 1;
        } else {
            break;
        }
    }
    i
}

/// Safely slice a string at byte offsets, respecting UTF-8 boundaries
fn safe_slice(s: &str, start: usize, end: usize) -> String {
    let end = end.min(s.len());
    let start = start.min(end);
    // Walk backward/forward to a char boundary if needed
    let mut actual_start = start;
    while actual_start > 0 && !s.is_char_boundary(actual_start) {
        actual_start -= 1;
    }
    let mut actual_end = end;
    while actual_end < s.len() && !s.is_char_boundary(actual_end) {
        actual_end += 1;
    }
    s[actual_start..actual_end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(cmd: &str) -> Vec<ShellExpansion> {
        ExpansionDetector::new(ShellType::Bash).detect(cmd)
    }

    #[test]
    fn detects_command_substitution() {
        let r = detect("echo $(whoami)");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::CommandSubstitution);
        assert!(r[0].description.contains("$(whoami)"));
    }

    #[test]
    fn detects_nested_command_substitution() {
        let r = detect("echo $(ls $(pwd))");
        // Outer $(...) captures everything; we report the outer one
        assert!(r
            .iter()
            .any(|e| e.kind == ExpansionKind::CommandSubstitution));
    }

    #[test]
    fn detects_backtick_substitution() {
        let r = detect("echo `date`");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::BacktickSubstitution);
    }

    #[test]
    fn detects_parameter_expansion() {
        let r = detect("echo ${HOME}");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::ParameterExpansion);
    }

    #[test]
    fn detects_parameter_expansion_with_default() {
        let r = detect("echo ${FOO:-bar}");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::ParameterExpansion);
    }

    #[test]
    fn detects_simple_variable() {
        let r = detect("echo $HOME");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::VariableReference);
    }

    #[test]
    fn detects_process_substitution() {
        let r = detect("diff <(sort a) <(sort b)");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].kind, ExpansionKind::ProcessSubstitution);
        assert_eq!(r[1].kind, ExpansionKind::ProcessSubstitution);
    }

    #[test]
    fn detects_arithmetic_expansion() {
        let r = detect("echo $((1 + 2))");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::ArithmeticExpansion);
    }

    #[test]
    fn single_quoted_expansion_is_literal() {
        // Single-quoted strings are literal on POSIX shells
        let r = detect("echo '$(rm -rf /)'");
        assert_eq!(r.len(), 0, "single-quoted expansions must not be detected");
    }

    #[test]
    fn double_quoted_expansion_is_detected() {
        // Double-quoted strings still expand
        let r = detect("echo \"$(whoami)\"");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::CommandSubstitution);
    }

    #[test]
    fn double_quoted_variable_is_detected() {
        let r = detect("echo \"$HOME\"");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].kind, ExpansionKind::VariableReference);
    }

    #[test]
    fn safe_command_has_no_expansions() {
        let r = detect("ls -la /tmp");
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn escaped_dollar_not_expansion() {
        let r = detect("echo \\$HOME");
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn dangerous_command_in_substitution() {
        let r = detect("echo $(rm -rf /)");
        assert_eq!(r.len(), 1);
        assert!(r[0].kind.executes_command());
    }

    #[test]
    fn powershell_skipped() {
        let d = ExpansionDetector::new(ShellType::PowerShell);
        // PowerShell has its own syntax; we skip it
        assert_eq!(d.detect("echo $(whoami)").len(), 0);
    }

    #[test]
    fn executes_command_classifier() {
        assert!(ExpansionKind::CommandSubstitution.executes_command());
        assert!(ExpansionKind::BacktickSubstitution.executes_command());
        assert!(ExpansionKind::ProcessSubstitution.executes_command());
        assert!(!ExpansionKind::VariableReference.executes_command());
        assert!(!ExpansionKind::ParameterExpansion.executes_command());
        assert!(!ExpansionKind::ArithmeticExpansion.executes_command());
    }

    #[test]
    fn multiple_expansions_in_order() {
        let r = detect("echo $FOO $(date) ${BAR}");
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].kind, ExpansionKind::VariableReference);
        assert_eq!(r[1].kind, ExpansionKind::CommandSubstitution);
        assert_eq!(r[2].kind, ExpansionKind::ParameterExpansion);
    }

    #[test]
    fn unmatched_paren_does_not_panic() {
        // Malformed input should not panic
        let r = detect("echo $(unmatched");
        // No complete expansion to report
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn find_paren_nested() {
        let bytes = b"(a(b)c)";
        assert_eq!(find_matching_paren(bytes, 0), Some(6));
    }

    #[test]
    fn find_paren_simple() {
        let bytes = b"(abc)";
        assert_eq!(find_matching_paren(bytes, 0), Some(4));
    }

    #[test]
    fn find_paren_unmatched() {
        let bytes = b"(abc";
        assert_eq!(find_matching_paren(bytes, 0), None);
    }
}
