// Deterministic context sanitizer for the hybrid privacy gateway.
//
// Before a prompt is sent to a remote inference network (Mesh-LLM, AI-Horde),
// the hybrid gateway runs it through this sanitizer to replace personally
// identifying or environment-revealing tokens with reversible placeholders
// (e.g. `/Users/alice/secret.txt` -> `<PATH_1>`). After the remote returns a
// command, the SAME session restores the real values locally, so the network
// never sees PII but the executed command is still correct.
//
// Design guarantees:
//   * Deterministic   - the same input always yields the same placeholders, so
//                       output is reproducible and cache-safe. No LLM is used;
//                       redaction is pure rule/regex based.
//   * Reversible      - every placeholder maps back to exactly one original.
//   * Class ordering  - broader classes (paths) are redacted before narrower
//                       ones (usernames) so a username inside a path is not
//                       half-leaked.
//
// Redaction scope ("Broad"): emails, IPv4 addresses, absolute/home paths,
// the current username and hostname (exact-literal), and the values of
// uppercase ENV-style assignments (`AWS_SECRET=...`).

use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").unwrap());

static IPV4_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()
});

// Absolute (`/usr/...`) or home (`~/...`) paths: a leading `/` or `~/` followed
// by at least one path-ish character. Requires the second char so a lone `/`
// (e.g. a regex slash in NL) is not redacted.
static PATH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:~/|/)[A-Za-z0-9._\-]+(?:/[A-Za-z0-9._\-]+)*/?").unwrap());

// Uppercase ENV-style assignment; the *value* (group 2) is redacted, the name
// (group 1) is kept so the command still reads sensibly.
static ENV_ASSIGN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([A-Z][A-Z0-9_]{2,})=(\S+)").unwrap());

/// Builds sanitizing sessions seeded with known identity literals.
#[derive(Debug, Clone, Default)]
pub struct ContextSanitizer {
    /// Known exact-string identifiers to redact, as `(value, class)` pairs —
    /// e.g. `("alice", "USER")`. Only non-empty, length-3+ values are used to
    /// avoid redacting trivially-short or empty identifiers.
    literals: Vec<(String, &'static str)>,
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
                self.literals.push((u.to_string(), "USER"));
            }
        }
        if let Some(h) = hostname {
            if h.len() >= 3 {
                self.literals.push((h.to_string(), "HOST"));
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
}

/// An in-flight sanitization with a reversible placeholder map.
pub struct SanitizeSession<'a> {
    literals: &'a [(String, &'static str)],
    /// `(placeholder, original)` in allocation order.
    entries: Vec<(String, String)>,
    /// Per-class running counters: `(class, next_index)`.
    counters: Vec<(&'static str, usize)>,
}

impl SanitizeSession<'_> {
    /// Redact PII from `text`, returning the placeholdered string.
    pub fn sanitize(&mut self, text: &str) -> String {
        let mut out = text.to_string();

        // Order matters: most-specific / broadest-span classes first so inner
        // tokens (e.g. a username inside a path) are not separately leaked.
        out = self.redact_regex(&out, &EMAIL_RE, "EMAIL");
        out = self.redact_regex(&out, &IPV4_RE, "IP");
        out = self.redact_regex(&out, &PATH_RE, "PATH");
        out = self.redact_env_assignments(&out);
        out = self.redact_literals(&out);

        out
    }

    /// Restore real values in `command` (the remote-generated output).
    pub fn restore(&self, command: &str) -> String {
        // Replace longest placeholders first so `<PATH_1>` does not corrupt
        // `<PATH_11>`.
        let mut ordered: Vec<&(String, String)> = self.entries.iter().collect();
        ordered.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        let mut out = command.to_string();
        for (placeholder, original) in ordered {
            out = out.replace(placeholder.as_str(), original);
        }
        out
    }

    /// Number of distinct values redacted in this session.
    pub fn redaction_count(&self) -> usize {
        self.entries.len()
    }

    /// Allocate (or reuse) a placeholder for `value` in `class`.
    fn placeholder_for(&mut self, value: &str, class: &'static str) -> String {
        if let Some((ph, _)) = self.entries.iter().find(|(_, v)| v == value) {
            return ph.clone();
        }
        let idx = match self.counters.iter_mut().find(|(c, _)| *c == class) {
            Some((_, n)) => {
                *n += 1;
                *n
            }
            None => {
                self.counters.push((class, 1));
                1
            }
        };
        let ph = format!("<{}_{}>", class, idx);
        self.entries.push((ph.clone(), value.to_string()));
        ph
    }

    fn redact_regex(&mut self, text: &str, re: &Regex, class: &'static str) -> String {
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
            let ph = self.placeholder_for(&v, "ENV");
            // Only replace the value when it follows a `NAME=`, preserving name.
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
        let literals: Vec<(String, &'static str)> = self.literals.to_vec();
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
        assert!(out.contains("<PATH_1>"));
        assert!(!out.contains("alice"));
        assert!(!out.contains("secret.txt"));

        // A remote-generated command referencing the placeholder restores fully.
        let restored = s.restore("rm <PATH_1>");
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
        // Same path appears twice -> one placeholder reused.
        assert_eq!(out.matches("<PATH_1>").count(), 2);
        assert!(!out.contains("<PATH_2>"));
    }

    #[test]
    fn test_username_inside_path_not_separately_leaked() {
        let san = ContextSanitizer::new().with_identity(Some("alice"), None);
        let mut s = san.session();
        let out = s.sanitize("open /home/alice/notes and tell alice");
        // The path is redacted wholesale; the standalone "alice" becomes <USER_1>.
        assert!(out.contains("<PATH_1>"));
        assert!(out.contains("<USER_1>"));
        assert!(!out.contains("alice"));
    }

    #[test]
    fn test_email_and_ip() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("mail bob@example.com from 192.168.1.10");
        assert!(out.contains("<EMAIL_1>"));
        assert!(out.contains("<IP_1>"));
        assert!(!out.contains("bob@example.com"));
        assert!(!out.contains("192.168.1.10"));
    }

    #[test]
    fn test_env_assignment_keeps_name_redacts_value() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("run with AWS_SECRET=hunter2 set");
        assert!(out.contains("AWS_SECRET=<ENV_1>"));
        assert!(!out.contains("hunter2"));
    }

    #[test]
    fn test_shared_session_across_input_and_context() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let i = s.sanitize("clean /tmp/cache");
        let c = s.sanitize("cwd is /tmp/cache");
        // Same path in both fields -> identical placeholder.
        assert!(i.contains("<PATH_1>"));
        assert!(c.contains("<PATH_1>"));
        assert_eq!(s.redaction_count(), 1);
    }

    #[test]
    fn test_no_pii_passes_through_unchanged() {
        let san = ContextSanitizer::new();
        let mut s = san.session();
        let out = s.sanitize("list all text files by size");
        assert_eq!(out, "list all text files by size");
        assert_eq!(s.redaction_count(), 0);
    }
}
