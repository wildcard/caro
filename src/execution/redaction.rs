//! Output redaction post-filter.
//!
//! Strips well-known secret patterns (AWS keys, GitHub PATs, JWTs, Bearer
//! tokens, env-var-style `*_TOKEN` / `*_KEY` / `*_SECRET` assignments, PEM
//! private-key blocks) from a command's stdout/stderr **after** the command
//! has executed and **before** caro displays the output.
//!
//! ## Scope
//!
//! - Applies to *output*, never to the *generated command*. A user asking
//!   "show my AWS credentials" must still see the literal command in
//!   `--dry-run`; the redactor only intervenes on captured stdout/stderr.
//! - Fail-safe: integration goes through
//!   [`crate::execution::executor::CommandExecutor::apply_filter`], so a
//!   bug in the redactor cannot drop the user's output.
//! - Conservative defaults: patterns target high-specificity prefixes
//!   (`AKIA…`, `ghp_…`, `eyJ…`) to keep false positives low. Generic
//!   high-entropy heuristics are intentionally out of scope.
//!
//! ## Replacement marker
//!
//! Matches are replaced with the literal marker `[REDACTED:<kind>]` (e.g.
//! `[REDACTED:aws-access-key]`). This breaks byte-position dependencies in
//! pathological downstream tools, but those are rare; the labelled marker is
//! significantly more useful for debugging than fixed-length asterisks.
//!
//! Pattern idea-borrowed from rtk-ai/rtk's per-command secret stripping
//! (Apache-2.0); reimplemented as a generic post-filter in caro's idioms.

use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

use crate::execution::executor::{CommandExecutor, ExecutionResult};

/// A post-execution redactor. Implementers transform `text` and return the
/// possibly-modified result.
pub trait OutputRedactor: Send + Sync {
    /// Redact `text`. Implementations should return `Cow::Borrowed(text)`
    /// when no changes were made to avoid unnecessary allocations.
    fn redact<'a>(&self, text: &'a str) -> Cow<'a, str>;
}

/// One entry in the built-in pattern table.
struct PatternEntry {
    kind: &'static str,
    regex: Regex,
}

/// Built-in redaction patterns. Conservative by design — targets specific
/// prefixes and assignment shapes rather than generic high-entropy strings.
static BUILTIN_PATTERNS: Lazy<Vec<PatternEntry>> = Lazy::new(|| {
    let raw: &[(&str, &str)] = &[
        // AWS access key id — fixed 16-char suffix after `AKIA`/`ASIA`.
        ("aws-access-key", r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"),
        // GitHub personal access tokens (and friends).
        // ghp = personal, gho = oauth, ghu = user-to-server,
        // ghs = server-to-server, ghr = refresh.
        ("github-token", r"\bgh[pousr]_[A-Za-z0-9]{36,}\b"),
        // JSON Web Tokens (3 base64url segments). The leading `eyJ` is the
        // base64 of the literal `{"` that opens every JWT header.
        (
            "jwt",
            r"\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b",
        ),
        // HTTP Authorization Bearer tokens.
        ("bearer-token", r"(?i)\bbearer\s+[A-Za-z0-9._\-+/=]{20,}"),
        // PEM private key blocks (multi-line, anchored on both ends).
        (
            "pem-private-key",
            r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
        ),
        // Env-var-style assignments where the *key* hints at a secret.
        // Captures TOKEN, KEY, SECRET, PASSWORD, PASSWD. Value runs to
        // end-of-line / whitespace / quote.
        (
            "env-secret-assignment",
            r#"(?i)\b([A-Z][A-Z0-9_]*?(?:TOKEN|KEY|SECRET|PASSWORD|PASSWD))\s*[:=]\s*['"]?[^\s'"]{6,}['"]?"#,
        ),
    ];
    raw.iter()
        .map(|(kind, pat)| PatternEntry {
            kind,
            regex: Regex::new(pat).expect("built-in redaction pattern compiles"),
        })
        .collect()
});

/// Default redactor: uses the built-in pattern set.
#[derive(Default, Debug, Clone, Copy)]
pub struct PatternRedactor;

impl PatternRedactor {
    pub fn new() -> Self {
        Self
    }
}

impl OutputRedactor for PatternRedactor {
    fn redact<'a>(&self, text: &'a str) -> Cow<'a, str> {
        let mut current: Cow<'a, str> = Cow::Borrowed(text);
        for entry in BUILTIN_PATTERNS.iter() {
            let replacement = format!("[REDACTED:{}]", entry.kind);
            // env-secret-assignment keeps the key visible so the user knows
            // *what* was redacted, swapping only the value.
            let new_text: Cow<str> = if entry.kind == "env-secret-assignment" {
                entry
                    .regex
                    .replace_all(&current, format!("$1={}", replacement).as_str())
            } else {
                entry.regex.replace_all(&current, replacement.as_str())
            };
            if let Cow::Owned(s) = new_text {
                current = Cow::Owned(s);
            }
        }
        current
    }
}

/// Apply a redactor to an [`ExecutionResult`] via the fail-safe
/// [`CommandExecutor::apply_filter`] pipeline.
///
/// Returns a new result with `stdout`/`stderr` redacted. `exit_code`,
/// `success`, and `execution_time_ms` are always preserved. If the redactor
/// panics, the raw result passes through with a warning (see
/// `CommandExecutor::apply_filter` for the invariant).
pub fn redact_result<R: OutputRedactor>(result: ExecutionResult, redactor: &R) -> ExecutionResult {
    CommandExecutor::apply_filter(result, |r| {
        let new_stdout = redactor.redact(&r.stdout).into_owned();
        let new_stderr = redactor.redact(&r.stderr).into_owned();
        if new_stdout == r.stdout && new_stderr == r.stderr {
            return None;
        }
        Some(ExecutionResult {
            exit_code: r.exit_code,
            stdout: new_stdout,
            stderr: new_stderr,
            execution_time_ms: r.execution_time_ms,
            success: r.success,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r() -> PatternRedactor {
        PatternRedactor::new()
    }

    #[test]
    fn test_aws_access_key_redacted() {
        let s = "key=AKIAIOSFODNN7EXAMPLE end";
        let out = r().redact(s);
        assert!(out.contains("[REDACTED:aws-access-key]"));
        assert!(!out.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(out.contains("end"));
    }

    #[test]
    fn test_github_pat_redacted() {
        let s = "token=ghp_abcdefghijklmnopqrstuvwxyzABCDEFGHIJ done";
        let out = r().redact(s);
        assert!(out.contains("[REDACTED:github-token]"));
        assert!(!out.contains("ghp_abc"));
    }

    #[test]
    fn test_jwt_redacted() {
        // 3 base64url-looking segments separated by dots, each long enough.
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let out = r().redact(jwt);
        assert!(out.contains("[REDACTED:jwt]"));
        assert!(!out.contains("eyJhbGci"));
    }

    #[test]
    fn test_bearer_token_redacted() {
        let s = "Authorization: Bearer abcdefghijklmnopqrstuvwxyz1234567890";
        let out = r().redact(s);
        assert!(out.contains("[REDACTED:bearer-token]"));
        assert!(!out.contains("abcdefghijklmnopqrstuvwxyz1234567890"));
    }

    #[test]
    fn test_pem_private_key_redacted() {
        let pem = "before\n-----BEGIN RSA PRIVATE KEY-----\nMIIE...\nasdf\n-----END RSA PRIVATE KEY-----\nafter";
        let out = r().redact(pem);
        assert!(out.contains("before"));
        assert!(out.contains("[REDACTED:pem-private-key]"));
        assert!(out.contains("after"));
        assert!(!out.contains("MIIE"));
    }

    #[test]
    fn test_env_secret_assignment_keeps_key_visible() {
        let s = "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let out = r().redact(s);
        // Key name should remain so the user knows *what* was redacted.
        assert!(out.contains("AWS_SECRET_ACCESS_KEY"));
        assert!(out.contains("[REDACTED:env-secret-assignment]"));
        assert!(!out.contains("wJalrXUtn"));
    }

    #[test]
    fn test_env_assignment_token_variant_matches() {
        let s = "GITHUB_TOKEN=ghp_abcdefghij1234567890ABCDEFG";
        let out = r().redact(s);
        // Either env-secret-assignment OR github-token fires — both are fine.
        assert!(out.contains("[REDACTED"));
        assert!(!out.contains("ghp_abcdefghij1234567890ABCDEFG"));
    }

    #[test]
    fn test_unrelated_text_is_unchanged() {
        let s = "Hello world, just a normal line with numbers 12345 and words.";
        let out = r().redact(s);
        // Borrowed Cow means no changes happened.
        assert!(matches!(out, Cow::Borrowed(_)));
        assert_eq!(out, s);
    }

    #[test]
    fn test_short_strings_below_threshold_are_not_matched() {
        // Bearer tokens require ≥20 chars to avoid false positives.
        let s = "Bearer short";
        let out = r().redact(s);
        assert_eq!(out, s);
    }

    #[test]
    fn test_redact_result_preserves_exit_code_and_timing() {
        let original = ExecutionResult {
            exit_code: 7,
            stdout: "key=AKIAIOSFODNN7EXAMPLE here".into(),
            stderr: "Bearer abcdefghijklmnopqrstuvwxyz1234567890".into(),
            execution_time_ms: 42,
            success: false,
        };
        let red = redact_result(original, &PatternRedactor::new());
        assert_eq!(red.exit_code, 7);
        assert_eq!(red.execution_time_ms, 42);
        assert!(!red.success);
        assert!(red.stdout.contains("[REDACTED:aws-access-key]"));
        assert!(red.stderr.contains("[REDACTED:bearer-token]"));
    }

    #[test]
    fn test_redact_result_pass_through_when_no_match() {
        let original = ExecutionResult {
            exit_code: 0,
            stdout: "boring output".into(),
            stderr: "boring stderr".into(),
            execution_time_ms: 1,
            success: true,
        };
        let red = redact_result(original.clone(), &PatternRedactor::new());
        assert_eq!(red.stdout, original.stdout);
        assert_eq!(red.stderr, original.stderr);
        assert_eq!(red.exit_code, original.exit_code);
    }
}
