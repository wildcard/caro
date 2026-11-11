//! PostToolUse secret-leak scan (spec 011 §P2.5).
//!
//! After a tool runs, the harness sends its output through the same hook.
//! This module looks for credential shapes in that output and reports the
//! **class** of what it found, never the matched text. Detection only: a
//! PostToolUse hook cannot block in Claude Code, and Codex rejects
//! `additionalContext`, so the result goes to the decision log and to a
//! stderr note. Enforce behavior is deferred until the shadow log shows how
//! often this fires.

use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

/// Largest tool response that is scanned; the rest is ignored, not failed.
pub const MAX_SCAN_BYTES: usize = 1024 * 1024;

/// One credential shape and the label recorded when it matches.
struct Shape {
    label: &'static str,
    re: Regex,
}

fn shape(label: &'static str, pattern: &str) -> Shape {
    Shape {
        label,
        re: RegexBuilder::new(pattern)
            .size_limit(1 << 20)
            .build()
            .expect("valid leak regex"),
    }
}

static SHAPES: Lazy<Vec<Shape>> = Lazy::new(|| {
    vec![
        shape("AWS access key id", r"\bAKIA[0-9A-Z]{16}\b"),
        shape(
            "private key block",
            r"-----BEGIN (?:(?:RSA |EC |DSA |OPENSSH |ENCRYPTED )?PRIVATE KEY|PGP PRIVATE KEY BLOCK)-----",
        ),
        shape("GitHub token", r"\bgh[pousr]_[A-Za-z0-9]{36,}\b"),
        shape(
            "GitHub fine-grained token",
            r"\bgithub_pat_[A-Za-z0-9_]{60,}\b",
        ),
        shape("Slack token", r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"),
        // Anthropic keys share the `sk-` prefix; they are matched first and
        // excluded from the OpenAI shape so one key counts once.
        shape("Anthropic API key", r"\bsk-ant-[A-Za-z0-9_-]{32,}\b"),
        shape(
            "OpenAI-style API key",
            r"\bsk-(?:proj-)?[A-Za-z0-9_-]{32,}\b",
        ),
        shape("Google API key", r"\bAIza[0-9A-Za-z_-]{35}\b"),
        shape(
            "Stripe secret key",
            r"\b[sr]k_(?:live|test)_[A-Za-z0-9]{20,}\b",
        ),
        shape(
            "JSON Web Token",
            r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b",
        ),
        shape(
            "authorization header",
            r#"(?i)\bauthorization\s*:\s*(?:bearer|basic|token)\s+[A-Za-z0-9._~+/=-]{16,}"#,
        ),
        shape(
            "secret-named assignment",
            r#"(?i)\b[A-Z0-9_]*(?:SECRET|TOKEN|PASSWORD|PASSWD|API_KEY|ACCESS_KEY|PRIVATE_KEY)[A-Z0-9_]*\s*[=:]\s*["']?[^\s"']{8,}"#,
        ),
    ]
});

/// Scan tool output for credential shapes. Returns the distinct labels that
/// matched, in shape order. The matched text is never returned.
pub fn scan(text: &str) -> Vec<&'static str> {
    let text = if text.len() > MAX_SCAN_BYTES {
        // Cut on a char boundary so the slice is valid UTF-8.
        let mut end = MAX_SCAN_BYTES;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    } else {
        text
    };
    let mut hits: Vec<&'static str> = SHAPES
        .iter()
        .filter(|s| s.re.is_match(text))
        .map(|s| s.label)
        .collect();
    // An Anthropic key also satisfies the broader `sk-` shape; count it once.
    if hits.contains(&"Anthropic API key") {
        hits.retain(|h| *h != "OpenAI-style API key");
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_shapes_without_returning_them() {
        // Fixtures are assembled at runtime so no credential-shaped literal
        // lives in the source.
        let aws = format!("AKIA{}", "EXAMPLEKEY123456");
        let gh = format!("ghp_{}", "a".repeat(36));
        let pem = ["-----BEGIN ", "PRIVATE KEY-----"].concat();
        let out = format!("deploy ok\n{aws}\n{gh}\n{pem}\nexport DB_PASSWORD=hunter22hunter\n");
        let hits = scan(&out);
        assert!(hits.contains(&"AWS access key id"), "{hits:?}");
        assert!(hits.contains(&"GitHub token"), "{hits:?}");
        assert!(hits.contains(&"private key block"), "{hits:?}");
        assert!(hits.contains(&"secret-named assignment"), "{hits:?}");
        for h in &hits {
            assert!(
                !h.contains("AKIA") && !h.contains("ghp_"),
                "label leaks text: {h}"
            );
        }
    }

    #[test]
    fn pgp_block_header_and_key_prefix_disambiguation() {
        let pgp = ["-----BEGIN PGP ", "PRIVATE KEY BLOCK-----"].concat();
        assert_eq!(scan(&pgp), vec!["private key block"]);
        let ant = format!("sk-ant-{}", "a".repeat(40));
        assert_eq!(scan(&ant), vec!["Anthropic API key"], "counted once");
        let oai = format!("sk-{}", "b".repeat(40));
        assert_eq!(scan(&oai), vec!["OpenAI-style API key"]);
        let proj = format!("sk-proj-{}", "c".repeat(40));
        assert_eq!(scan(&proj), vec!["OpenAI-style API key"]);
    }

    #[test]
    fn routine_output_is_clean() {
        for out in [
            "total 48\ndrwxr-xr-x  12 user  staff   384 Oct 18 20:00 .\n",
            "Compiling caro v1.5.0\n    Finished `dev` profile",
            "token count: 1234\npassword prompt disabled\n",
            "PATH=/usr/bin:/bin\nHOME=/home/user\n",
            "Authorization: see docs",
        ] {
            assert!(scan(out).is_empty(), "{out}");
        }
    }

    #[test]
    fn oversized_output_is_truncated_not_rejected() {
        let mut big = "x".repeat(MAX_SCAN_BYTES + 10);
        big.push_str(&format!("AKIA{}", "EXAMPLEKEY123456"));
        // The key sits past the cap, so it is not seen; the scan still runs.
        assert!(scan(&big).is_empty());
        let mut front = format!("AKIA{} ", "EXAMPLEKEY123456");
        front.push_str(&"y".repeat(MAX_SCAN_BYTES + 10));
        assert_eq!(scan(&front), vec!["AWS access key id"]);
    }
}
