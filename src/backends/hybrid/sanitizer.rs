// Deterministic context sanitizer for the hybrid privacy gateway.
//
// Before a prompt is sent to a remote inference network (Mesh-LLM, AI-Horde),
// the hybrid gateway runs it through this sanitizer to replace personally
// identifying or environment-revealing tokens with TYPED, self-describing
// placeholders (e.g. `/Users/alice/secret.txt` -> `<REDACTED_FILEPATH_1>`).
// After the remote returns a command, the SAME session restores the real
// values locally, so the network never sees PII but the executed command is
// still correct.
//
// Crucially, the remote model is not left to guess what a placeholder means.
// `redaction_briefing()` produces a legend describing each placeholder ("an
// absolute filesystem path", "the user's login name", ...) plus an explicit
// instruction stating that Caro's local model performed the redaction on the
// user's machine and that the placeholder must be reproduced verbatim. This
// lets the remote reason about the *shape* of the command without ever seeing
// the private value.
//
// Design guarantees:
//   * Deterministic   - the same input always yields the same placeholders, so
//                       output is reproducible and cache-safe. No LLM is used;
//                       redaction is pure rule/regex based.
//   * Reversible      - every placeholder maps back to exactly one original.
//   * Class ordering  - broader classes (paths) are redacted before narrower
//                       ones (usernames) so a username inside a path is not
//                       half-leaked.
//   * Self-describing - each placeholder carries a class and a human/LLM
//                       readable description for the briefing legend.

use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").unwrap());

static IPV4_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap());

// Absolute (`/usr/...`) or home (`~/...`) paths: a leading `/` or `~/` followed
// by at least one path-ish character. Requires the second char so a lone `/`
// (e.g. a regex slash in NL) is not redacted.
static PATH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:~/|/)[A-Za-z0-9._\-]+(?:/[A-Za-z0-9._\-]+)*/?").unwrap());

// Uppercase ENV-style assignment; the *value* (group 2) is redacted, the name
// (group 1) is kept so the command still reads sensibly.
static ENV_ASSIGN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([A-Z][A-Z0-9_]{2,})=(\S+)").unwrap());

/// A redaction class: the token stem used in placeholders and a description of
/// what kind of value it stands for (shown to the remote model in the legend).
#[derive(Debug, Clone, Copy)]
struct RedactionClass {
    /// Token stem, e.g. `REDACTED_FILEPATH` -> `<REDACTED_FILEPATH_1>`.
    token: &'static str,
    /// Human/LLM-readable description for the briefing legend.
    description: &'static str,
}

const CLASS_EMAIL: RedactionClass = RedactionClass {
    token: "REDACTED_EMAIL",
    description: "an email address",
};
const CLASS_IPV4: RedactionClass = RedactionClass {
    token: "REDACTED_IPV4",
    description: "an IPv4 network address",
};
const CLASS_FILEPATH: RedactionClass = RedactionClass {
    token: "REDACTED_FILEPATH",
    description: "an absolute or home-directory filesystem path on the user's machine",
};
const CLASS_ENV_VALUE: RedactionClass = RedactionClass {
    token: "REDACTED_ENV_VALUE",
    description: "the value of an environment variable",
};
const CLASS_USERNAME: RedactionClass = RedactionClass {
    token: "REDACTED_USERNAME",
    description: "the user's account / login name",
};
const CLASS_HOSTNAME: RedactionClass = RedactionClass {
    token: "REDACTED_HOSTNAME",
    description: "the machine's hostname",
};

/// One redacted value and the placeholder that replaced it.
#[derive(Debug, Clone)]
struct Redaction {
    placeholder: String,
    original: String,
    description: &'static str,
}

/// Builds sanitizing sessions seeded with known identity literals.
#[derive(Debug, Clone, Default)]
pub struct ContextSanitizer {
    /// Known exact-string identifiers to redact, as `(value, class)` pairs.
    /// Only non-empty, length-3+ values are kept to avoid redacting trivially
    /// short identifiers.
    literals: Vec<(String, RedactionClass)>,
}

impl ContextSanitizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed the sanitizer with the current username and hostname so those exact
    /// strings are redacted wherever they appear in the prompt.
    pub fn with_identity(mut self, username: Option<&str>, hostname: Option<&str>) -> Self {
        if let Some(u) = username {
            if u.len() >= 3 {
                self.literals.push((u.to_string(), CLASS_USERNAME));
            }
        }
        if let Some(h) = hostname {
            if h.len() >= 3 {
                self.literals.push((h.to_string(), CLASS_HOSTNAME));
            }
        }
        self
    }

    /// Start a sanitizing session. A session accumulates the placeholder map
    /// across multiple `sanitize` calls (e.g. input + context) so a value gets
    /// one stable placeholder, and can later `restore` it in the result.
    pub fn session(&self) -> SanitizeSession<'_> {
        SanitizeSession {
            literals: &self.literals,
            entries: Vec::new(),
            counters: Vec::new(),
        }
    }

    /// A standing note attached to the LOCAL model's context whenever the
    /// hybrid gateway runs, so the on-device model is an aware participant in
    /// the redaction process (not a bystander). The local model owns redaction;
    /// this states the contract it operates under.
    pub fn local_awareness_note() -> &'static str {
        "[caro-privacy-layer] You are Caro's on-device model. A deterministic \
local redaction layer rewrites private values (filesystem paths, usernames, \
hostnames, IP addresses, emails, and environment-variable values) into typed \
placeholders before any prompt is sent to a remote backend, and restores them \
locally afterwards. Private values never leave this machine."
    }
}

/// An in-flight sanitization with a reversible placeholder map.
pub struct SanitizeSession<'a> {
    literals: &'a [(String, RedactionClass)],
    /// Redactions in allocation order.
    entries: Vec<Redaction>,
    /// Per-class-token running counters: `(token, next_index)`.
    counters: Vec<(&'static str, usize)>,
}

impl SanitizeSession<'_> {
    /// Redact PII from `text`, returning the placeholdered string.
    pub fn sanitize(&mut self, text: &str) -> String {
        let mut out = text.to_string();

        // Order matters: broadest-span classes first so inner tokens (e.g. a
        // username inside a path) are not separately leaked.
        out = self.redact_regex(&out, &EMAIL_RE, CLASS_EMAIL);
        out = self.redact_regex(&out, &IPV4_RE, CLASS_IPV4);
        out = self.redact_regex(&out, &PATH_RE, CLASS_FILEPATH);
        out = self.redact_env_assignments(&out);
        out = self.redact_literals(&out);

        out
    }

    /// Restore real values in `command` (the remote-generated output).
    pub fn restore(&self, command: &str) -> String {
        // Replace longest placeholders first so `<REDACTED_FILEPATH_1>` does not
        // corrupt `<REDACTED_FILEPATH_11>`.
        let mut ordered: Vec<&Redaction> = self.entries.iter().collect();
        ordered.sort_by(|a, b| b.placeholder.len().cmp(&a.placeholder.len()));
        let mut out = command.to_string();
        for r in ordered {
            out = out.replace(r.placeholder.as_str(), &r.original);
        }
        out
    }

    /// Number of distinct values redacted in this session.
    pub fn redaction_count(&self) -> usize {
        self.entries.len()
    }

    /// A briefing for the REMOTE model: states that Caro's local model performed
    /// the redaction on the harness, lists each placeholder with a description
    /// of the value it stands for, and instructs the model to reproduce the
    /// placeholder verbatim. Returns `None` when nothing was redacted.
    pub fn redaction_briefing(&self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let mut legend = String::new();
        for r in &self.entries {
            legend.push_str(&format!("  - {}: {}\n", r.placeholder, r.description));
        }

        Some(format!(
            "[CARO PRIVACY LAYER — read carefully]\n\
This request was preprocessed on the user's machine by Caro's local model. \
Sensitive values were redacted and replaced with the typed placeholders below. \
Each placeholder stands for a real value that never left the user's device:\n\
{legend}\
Treat every placeholder as an opaque literal: reproduce it EXACTLY as written \
in your generated command, and never guess or invent the underlying value. \
Caro will substitute the real values back locally before the command runs."
        ))
    }

    /// Allocate (or reuse) a placeholder for `value` in `class`.
    fn placeholder_for(&mut self, value: &str, class: RedactionClass) -> String {
        if let Some(r) = self.entries.iter().find(|r| r.original == value) {
            return r.placeholder.clone();
        }
        let idx = match self.counters.iter_mut().find(|(t, _)| *t == class.token) {
            Some((_, n)) => {
                *n += 1;
                *n
            }
            None => {
                self.counters.push((class.token, 1));
                1
            }
        };
        let placeholder = format!("<{}_{}>", class.token, idx);
        self.entries.push(Redaction {
            placeholder: placeholder.clone(),
            original: value.to_string(),
            description: class.description,
        });
        placeholder
    }

    fn redact_regex(&mut self, text: &str, re: &Regex, class: RedactionClass) -> String {
        // Collect distinct matches in order of first appearance.
        let mut distinct: Vec<String> = Vec::new();
        for m in re.find_iter(text) {
            let v = m.as_str().to_string();
            if !distinct.contains(&v) {
                distinct.push(v);
            }
        }
        let mut out = text.to_string();
        for v in distinct {
            let ph = self.placeholder_for(&v, class);
            out = out.replace(&v, &ph);
        }
        out
    }

    fn redact_env_assignments(&mut self, text: &str) -> String {
        let mut distinct: Vec<String> = Vec::new();
        for caps in ENV_ASSIGN_RE.captures_iter(text) {
            let v = caps.get(2).unwrap().as_str().to_string();
            if !distinct.contains(&v) {
                distinct.push(v);
            }
        }
        let mut out = text.to_string();
        for v in distinct {
            let ph = self.placeholder_for(&v, CLASS_ENV_VALUE);
            out = ENV_ASSIGN_RE
                .replace_all(&out, |caps: &regex::Captures| {
                    if caps.get(2).map(|m| m.as_str()) == Some(v.as_str()) {
                        format!("{}={}", &caps[1], ph)
                    } else {
                        caps[0].to_string()
                    }
                })
                .into_owned();
        }
        out
    }

    fn redact_literals(&mut self, text: &str) -> String {
        let mut out = text.to_string();
        // Snapshot to avoid borrowing self while mutating its map.
        let literals: Vec<(String, RedactionClass)> = self.literals.to_vec();
        for (value, class) in literals {
            if !value.is_empty() && out.contains(&value) {
                let ph = self.placeholder_for(&value, class);
                out = out.replace(&value, &ph);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_redaction_and_restore() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("delete /Users/alice/secret.txt please");
        assert!(out.contains("<REDACTED_FILEPATH_1>"));
        assert!(!out.contains("alice"));
        assert!(!out.contains("secret.txt"));

        let restored = s.restore("rm <REDACTED_FILEPATH_1>");
        assert_eq!(restored, "rm /Users/alice/secret.txt");
    }

    #[test]
    fn test_determinism() {
        let san = ContextSanitizer::new();
        let a = san.session().sanitize("look in /var/log/app and /etc/hosts");
        let b = san.session().sanitize("look in /var/log/app and /etc/hosts");
        assert_eq!(a, b, "same input must yield identical placeholders");
    }

    #[test]
    fn test_repeated_value_same_placeholder() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("copy /tmp/a to backup, then read /tmp/a again");
        assert_eq!(out.matches("<REDACTED_FILEPATH_1>").count(), 2);
        assert!(!out.contains("<REDACTED_FILEPATH_2>"));
    }

    #[test]
    fn test_username_inside_path_not_separately_leaked() {
        let san = ContextSanitizer::new().with_identity(Some("alice"), None);
        let mut s = san.session();
        let out = s.sanitize("open /home/alice/notes and tell alice");
        assert!(out.contains("<REDACTED_FILEPATH_1>"));
        assert!(out.contains("<REDACTED_USERNAME_1>"));
        assert!(!out.contains("alice"));
    }

    #[test]
    fn test_email_and_ip() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("mail bob@example.com from 192.168.1.10");
        assert!(out.contains("<REDACTED_EMAIL_1>"));
        assert!(out.contains("<REDACTED_IPV4_1>"));
        assert!(!out.contains("bob@example.com"));
        assert!(!out.contains("192.168.1.10"));
    }

    #[test]
    fn test_env_assignment_keeps_name_redacts_value() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("run with AWS_SECRET=hunter2 set");
        assert!(out.contains("AWS_SECRET=<REDACTED_ENV_VALUE_1>"));
        assert!(!out.contains("hunter2"));
    }

    #[test]
    fn test_shared_session_across_input_and_context() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let i = s.sanitize("clean /tmp/cache");
        let c = s.sanitize("cwd is /tmp/cache");
        assert!(i.contains("<REDACTED_FILEPATH_1>"));
        assert!(c.contains("<REDACTED_FILEPATH_1>"));
        assert_eq!(s.redaction_count(), 1);
    }

    #[test]
    fn test_no_pii_passes_through_unchanged() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("list all text files by size");
        assert_eq!(out, "list all text files by size");
        assert_eq!(s.redaction_count(), 0);
        assert!(s.redaction_briefing().is_none());
    }

    #[test]
    fn test_briefing_describes_each_placeholder_and_attributes_local_model() {
        let san = ContextSanitizer::new().with_identity(Some("alice"), None);
        let mut s = san.session();
        s.sanitize("tar /Users/alice/photos and mail bob@example.com");
        let briefing = s.redaction_briefing().expect("redactions present");

        // Attribution + verbatim instruction.
        assert!(briefing.contains("Caro's local model"));
        assert!(briefing.to_lowercase().contains("exactly"));
        // Legend entries carry descriptions, not just the bare token.
        assert!(briefing.contains("<REDACTED_FILEPATH_1>"));
        assert!(briefing.contains("filesystem path"));
        assert!(briefing.contains("<REDACTED_EMAIL_1>"));
        assert!(briefing.contains("email address"));
    }

    #[test]
    fn test_local_awareness_note_mentions_redaction() {
        let note = ContextSanitizer::local_awareness_note();
        assert!(note.contains("caro-privacy-layer"));
        assert!(note.to_lowercase().contains("redaction"));
    }
}
