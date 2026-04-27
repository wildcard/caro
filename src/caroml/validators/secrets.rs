//! Secrets angle — regex scan for hard-coded credentials in commands.
//!
//! Runs a tiny set of high-precision patterns. v0.1 is intentionally
//! conservative: false-positives are worse than misses since this is a
//! warn-only gate (it doesn't block generation, just flags for human review).
//!
//! The plan defers stdout secret scanning to v0.2 (it requires the runtime
//! journal to be feeding back, which arrives in PR 6).

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::caroml::validators::{ValidationOutcome, Validator, ValidatorContext, Verdict};

pub struct SecretsAngle;

impl Default for SecretsAngle {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Validator for SecretsAngle {
    fn angle(&self) -> &'static str {
        "secrets"
    }

    async fn validate(&self, ctx: &ValidatorContext<'_>) -> ValidationOutcome {
        for (kind, pattern) in PATTERNS.iter() {
            if pattern.is_match(ctx.command) {
                return ValidationOutcome {
                    angle: "secrets".to_string(),
                    result: Verdict::Warn,
                    note: Some(format!("possible {} in command", kind)),
                    repair_hint: Some(format!(
                        "move the {} into an env var or a credential file; \
                         reference it via $VAR rather than hard-coding",
                        kind
                    )),
                };
            }
        }
        ValidationOutcome::pass("secrets")
    }
}

/// (Human-readable kind, regex). Keep tight — high precision over high recall.
static PATTERNS: Lazy<Vec<(&'static str, Regex)>> = Lazy::new(|| {
    vec![
        ("AWS access key", Regex::new(r"AKIA[0-9A-Z]{16}").unwrap()),
        (
            "GitHub personal access token",
            Regex::new(r"\bghp_[A-Za-z0-9]{36}\b").unwrap(),
        ),
        (
            "GitHub fine-grained token",
            Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{82}\b").unwrap(),
        ),
        (
            "Slack token",
            Regex::new(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b").unwrap(),
        ),
        (
            "Stripe live key",
            Regex::new(r"\bsk_live_[A-Za-z0-9]{24,}\b").unwrap(),
        ),
        (
            "OpenAI API key",
            Regex::new(r"\bsk-[A-Za-z0-9]{20,}T3BlbkFJ[A-Za-z0-9]{20,}\b").unwrap(),
        ),
        (
            "private key block",
            Regex::new(r"-----BEGIN ([A-Z ]+)PRIVATE KEY-----").unwrap(),
        ),
        (
            "URL with embedded credentials",
            Regex::new(r"https?://[^/\s:@]+:[^/\s:@]+@").unwrap(),
        ),
    ]
});

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(command: &'a str) -> ValidatorContext<'a> {
        ValidatorContext {
            command,
            intent: "test",
            task_title: "test",
            platform: "linux",
            sudo_declared: false,
            capability_profile: None,
        }
    }

    #[tokio::test]
    async fn benign_command_passes() {
        let v = SecretsAngle;
        let outcome = v.validate(&ctx("curl https://example.com")).await;
        assert_eq!(outcome.result, Verdict::Pass);
    }

    #[tokio::test]
    async fn aws_access_key_warns() {
        let v = SecretsAngle;
        let outcome = v
            .validate(&ctx("export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE"))
            .await;
        assert_eq!(outcome.result, Verdict::Warn);
        assert!(outcome.note.unwrap().contains("AWS"));
    }

    #[tokio::test]
    async fn github_classic_token_warns() {
        let v = SecretsAngle;
        let outcome = v
            .validate(&ctx(
                "git push https://x-access-token:ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789@github.com/foo/bar.git",
            ))
            .await;
        assert_eq!(outcome.result, Verdict::Warn);
    }

    #[tokio::test]
    async fn url_with_basic_auth_warns() {
        let v = SecretsAngle;
        let outcome = v
            .validate(&ctx("curl https://admin:secret@example.com/api"))
            .await;
        assert_eq!(outcome.result, Verdict::Warn);
    }

    #[tokio::test]
    async fn private_key_block_warns() {
        let v = SecretsAngle;
        let outcome = v
            .validate(&ctx("echo '-----BEGIN OPENSSH PRIVATE KEY-----' > k"))
            .await;
        assert_eq!(outcome.result, Verdict::Warn);
    }
}
