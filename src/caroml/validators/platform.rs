//! Platform angle — heuristic checks for BSD vs GNU tool flags.
//!
//! v0.1 ships a small built-in heuristic table (no full POSIX coverage —
//! that's PR's beyond v1.0 territory). The goal is to flag the most common
//! cross-platform footguns: GNU-only flags on macOS, BSD-only flags on Linux,
//! and shell-specific bashisms in `/bin/sh` contexts.
//!
//! When a [`CapabilityProfile`] is attached to the [`ValidatorContext`] this
//! validator uses it for higher-confidence detection (e.g. probing the host's
//! `find -printf` support). Without a profile, it falls back to the platform
//! string and a static feature table.

use async_trait::async_trait;

use crate::caroml::validators::{ValidationOutcome, Validator, ValidatorContext, Verdict};

pub struct PlatformAngle;

impl Default for PlatformAngle {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Validator for PlatformAngle {
    fn angle(&self) -> &'static str {
        "platform"
    }

    async fn validate(&self, ctx: &ValidatorContext<'_>) -> ValidationOutcome {
        let cmd = ctx.command;

        // Macro-only shortcut: quick checks with no profile required.
        for (pattern, message, hint, restricted_to) in BUILTIN_HEURISTICS {
            if cmd.contains(*pattern) && platforms_match(restricted_to, ctx.platform) {
                return ValidationOutcome::warn(
                    "platform",
                    format!("{} (`{}`)", message, pattern),
                )
                .with_hint(*hint);
            }
        }

        // Capability-profile-driven checks (only when a profile is provided).
        if let Some(profile) = ctx.capability_profile {
            if cmd.contains("find ") && cmd.contains("-printf") && !profile.find_printf {
                return ValidationOutcome::fail(
                    "platform",
                    format!(
                        "`find -printf` is not available on this host ({})",
                        ctx.platform
                    ),
                    "use `find ... -exec stat ...` or `find ... | xargs stat ...` instead of -printf",
                );
            }
            if cmd.contains("sed -i ''") && profile.sed_inplace_gnu {
                return ValidationOutcome::warn(
                    "platform",
                    "GNU sed expects `-i` without an empty argument; \
                     `sed -i ''` is BSD/macOS syntax",
                )
                .with_hint("on Linux/GNU, use `sed -i` (no quoted empty string)");
            }
            if cmd.contains("sed -i\n") || cmd.contains("sed -i ") && !profile.sed_inplace_gnu {
                // BSD sed needs an explicit suffix arg, even if empty.
                if !cmd.contains("sed -i ''") && !cmd.contains("sed -i.") {
                    return ValidationOutcome::warn(
                        "platform",
                        "BSD/macOS `sed -i` requires an explicit backup-suffix argument",
                    )
                    .with_hint("on macOS, use `sed -i ''` (empty suffix) or `sed -i.bak`");
                }
            }
        }

        ValidationOutcome::pass("platform")
    }
}

/// Tiny extension to [`ValidationOutcome`] for builder-style hint attachment.
trait OutcomeExt {
    fn with_hint(self, hint: impl Into<String>) -> Self;
}

impl OutcomeExt for ValidationOutcome {
    fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.repair_hint = Some(hint.into());
        if self.result == Verdict::Pass {
            self.result = Verdict::Warn;
        }
        self
    }
}

/// Tuple: (pattern, human-readable message, repair hint, platforms to flag on).
type Heuristic = (&'static str, &'static str, &'static str, &'static [&'static str]);

const BUILTIN_HEURISTICS: &[Heuristic] = &[
    (
        "stat -c",
        "`stat -c` is GNU-only and not available in BSD",
        "use `stat -f` on macOS/BSD; or branch on `$OSTYPE`",
        &["macos"],
    ),
    (
        "stat -f",
        "`stat -f` is BSD/macOS-only and not available in GNU coreutils",
        "use `stat -c` on Linux; or branch on `$OSTYPE`",
        &["linux"],
    ),
    (
        "readlink -f",
        "`readlink -f` is GNU-only on older macOS; install coreutils or use `realpath`",
        "use `realpath` (or `python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' \"$path\"`)",
        &["macos"],
    ),
    (
        "useradd ",
        "`useradd` is Linux-only; macOS uses `dscl`",
        "branch on `$OSTYPE`; on macOS use `sudo dscl . -create /Users/<name>` etc.",
        &["macos"],
    ),
    (
        "apt ",
        "`apt` is Debian/Ubuntu-only",
        "use `brew` on macOS; `dnf`/`yum` on RHEL-family Linux",
        &["macos", "windows"],
    ),
    (
        "brew ",
        "`brew` is macOS / Linux-with-Homebrew only",
        "use the platform's native package manager (`apt`, `dnf`, `winget`, ...)",
        &["windows"],
    ),
];

fn platforms_match(restricted_to: &[&str], current: &str) -> bool {
    restricted_to.contains(&current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(command: &'a str, platform: &'a str) -> ValidatorContext<'a> {
        ValidatorContext {
            command,
            intent: "test",
            task_title: "test",
            platform,
            sudo_declared: false,
            capability_profile: None,
        }
    }

    #[tokio::test]
    async fn passes_clean_command() {
        let v = PlatformAngle;
        let outcome = v.validate(&ctx("ls -la /tmp", "macos")).await;
        assert_eq!(outcome.result, Verdict::Pass);
    }

    #[tokio::test]
    async fn flags_stat_dash_c_on_macos() {
        let v = PlatformAngle;
        let outcome = v.validate(&ctx("stat -c '%n' /tmp/foo", "macos")).await;
        assert_eq!(outcome.result, Verdict::Warn);
        assert!(outcome.note.unwrap().contains("GNU-only"));
    }

    #[tokio::test]
    async fn flags_stat_dash_f_on_linux() {
        let v = PlatformAngle;
        let outcome = v.validate(&ctx("stat -f '%N' /tmp/foo", "linux")).await;
        assert_eq!(outcome.result, Verdict::Warn);
    }

    #[tokio::test]
    async fn allows_apt_on_linux() {
        let v = PlatformAngle;
        let outcome = v.validate(&ctx("apt install jq", "linux")).await;
        assert_eq!(outcome.result, Verdict::Pass);
    }

    #[tokio::test]
    async fn flags_apt_on_macos() {
        let v = PlatformAngle;
        let outcome = v.validate(&ctx("apt install jq", "macos")).await;
        assert_eq!(outcome.result, Verdict::Warn);
        assert!(outcome.repair_hint.is_some());
    }

    #[tokio::test]
    async fn flags_brew_on_windows() {
        let v = PlatformAngle;
        let outcome = v.validate(&ctx("brew install jq", "windows")).await;
        assert_eq!(outcome.result, Verdict::Warn);
    }
}
