//! Side-effects angle — surfaces network, filesystem, and privilege effects.
//!
//! v0.1 is **warn-only**: this validator never blocks generation. It emits
//! informational notes that get surfaced via `caro why` and stored verbatim
//! in the lock's per-step `validations` array. v0.2 may add `must_pass`
//! gates for declared but unrequested side effects (e.g. a network call when
//! the task didn't `NEED network`).

use async_trait::async_trait;

use crate::caroml::validators::{ValidationOutcome, Validator, ValidatorContext, Verdict};

pub struct SideEffectsAngle;

impl Default for SideEffectsAngle {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Validator for SideEffectsAngle {
    fn angle(&self) -> &'static str {
        "side_effects"
    }

    async fn validate(&self, ctx: &ValidatorContext<'_>) -> ValidationOutcome {
        let cmd = ctx.command;
        let mut notes: Vec<String> = Vec::new();

        if has_sudo(cmd) {
            if !ctx.sudo_declared {
                notes.push(
                    "uses sudo; consider `NEED sudo` in the .caro to make this explicit".into(),
                );
            } else {
                notes.push("uses sudo (declared via NEED)".into());
            }
        }

        if has_network(cmd) {
            notes.push("makes network calls (curl/wget/ssh/scp/rsync over network)".into());
        }

        if has_destructive_fs(cmd) {
            notes.push("filesystem destructive: rm/rmdir/mv/truncate".into());
        }

        if has_systemwide_write(cmd) {
            notes.push(
                "writes outside the user's home — paths under /etc, /var, /usr, /opt, /System"
                    .into(),
            );
        }

        if notes.is_empty() {
            return ValidationOutcome::pass("side_effects");
        }

        ValidationOutcome {
            angle: "side_effects".to_string(),
            result: Verdict::Warn,
            note: Some(notes.join("; ")),
            repair_hint: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Heuristics
// ---------------------------------------------------------------------------

fn has_sudo(cmd: &str) -> bool {
    contains_word(cmd, "sudo") || contains_word(cmd, "doas")
}

fn has_network(cmd: &str) -> bool {
    [
        "curl ",
        "wget ",
        "scp ",
        "rsync ",
        "ssh ",
        "git push",
        "git pull",
        "git clone",
        "git fetch",
        "nc ",
        "ncat ",
        "telnet ",
        "openssl s_client",
    ]
    .iter()
    .any(|p| cmd.contains(p))
}

fn has_destructive_fs(cmd: &str) -> bool {
    contains_word(cmd, "rm")
        || contains_word(cmd, "rmdir")
        || contains_word(cmd, "truncate")
        || contains_word(cmd, "mv")
        || contains_word(cmd, "shred")
        || contains_word(cmd, "dd")
}

fn has_systemwide_write(cmd: &str) -> bool {
    [
        " /etc/", "> /etc/", " /var/", "> /var/", " /usr/", "> /usr/", " /opt/", "> /opt/",
        " /System/", "> /System/",
    ]
    .iter()
    .any(|p| cmd.contains(p))
}

/// Word-boundary contains: matches the bare token surrounded by whitespace, line
/// start/end, or simple punctuation (`;`, `&`, `|`, `(`, `)`).
fn contains_word(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let nbytes = needle.as_bytes();
    let mut i = 0;
    while i + nbytes.len() <= bytes.len() {
        if &bytes[i..i + nbytes.len()] == nbytes {
            let before_ok = i == 0 || is_boundary(bytes[i - 1]);
            let after = i + nbytes.len();
            let after_ok = after == bytes.len() || is_boundary(bytes[after]);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn is_boundary(b: u8) -> bool {
    matches!(
        b,
        b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b'(' | b')' | b'`'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(command: &'a str, sudo_declared: bool) -> ValidatorContext<'a> {
        ValidatorContext {
            command,
            intent: "test",
            task_title: "test",
            platform: "linux",
            sudo_declared,
            capability_profile: None,
        }
    }

    #[tokio::test]
    async fn read_only_command_passes() {
        let v = SideEffectsAngle;
        let outcome = v.validate(&ctx("ls -la /tmp", false)).await;
        assert_eq!(outcome.result, Verdict::Pass);
    }

    #[tokio::test]
    async fn sudo_without_declaration_warns_with_specific_note() {
        let v = SideEffectsAngle;
        let outcome = v.validate(&ctx("sudo apt update", false)).await;
        assert_eq!(outcome.result, Verdict::Warn);
        let note = outcome.note.unwrap();
        assert!(note.contains("NEED sudo"));
    }

    #[tokio::test]
    async fn declared_sudo_warns_but_acknowledges() {
        let v = SideEffectsAngle;
        let outcome = v.validate(&ctx("sudo apt update", true)).await;
        assert_eq!(outcome.result, Verdict::Warn);
        let note = outcome.note.unwrap();
        assert!(note.contains("declared via NEED"));
    }

    #[tokio::test]
    async fn network_command_warns() {
        let v = SideEffectsAngle;
        let outcome = v
            .validate(&ctx("curl https://example.com -o file.txt", false))
            .await;
        assert_eq!(outcome.result, Verdict::Warn);
        assert!(outcome.note.unwrap().contains("network"));
    }

    #[tokio::test]
    async fn rm_warns_as_destructive() {
        let v = SideEffectsAngle;
        let outcome = v.validate(&ctx("rm /tmp/foo", false)).await;
        assert_eq!(outcome.result, Verdict::Warn);
    }

    #[tokio::test]
    async fn write_to_etc_warns() {
        let v = SideEffectsAngle;
        let outcome = v
            .validate(&ctx("echo 'hello' > /etc/motd", false))
            .await;
        assert_eq!(outcome.result, Verdict::Warn);
    }

    #[tokio::test]
    async fn word_boundary_does_not_match_substring() {
        // "drum" should not match the word "rm"
        assert!(!contains_word("a drum kit", "rm"));
        // "rm" as a word should match
        assert!(contains_word("rm /tmp/foo", "rm"));
        assert!(contains_word("foo && rm bar", "rm"));
    }
}
