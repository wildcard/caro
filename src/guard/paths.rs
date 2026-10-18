//! Protected-path policy for file tools (spec 011 §P2.3).
//!
//! Harnesses expose `Read`, `Write`, `Edit`, `Grep` and `Glob` through the
//! same PreToolUse hook as `Bash`. This module decides, without touching the
//! filesystem, whether the path a file tool is about to use is one the human
//! should see first: credential files, key material, and caro's own guard
//! files. Matching is lexical: tilde and `.`/`..` are resolved, then the
//! path is compared against glob patterns. Symlinks are not followed (no
//! I/O on the hook's hot path).
//!
//! Policy: reading, searching or listing a protected path → `ask`; writing
//! or editing a protected path → `ask`; writing or editing caro's own
//! decision log → `deny`, because that is the one file the guard owns.
//! Everything else → no opinion. Caro never emits `allow`.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Verdict;

/// File tools across harnesses (compared lowercase).
pub const FILE_TOOLS: &[&str] = &[
    "read",
    "write",
    "edit",
    "multiedit",
    "notebookedit",
    "grep",
    "glob",
];

/// How a file tool touches its path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileAccess {
    /// `Read`, `Grep`, `Glob`: contents or names leave the file.
    Read,
    /// `Write`, `Edit`, `MultiEdit`, `NotebookEdit`: contents change.
    Write,
}

/// Classify a tool name, or `None` when it is not a file tool.
pub fn file_access(tool_name: &str) -> Option<FileAccess> {
    match tool_name.to_ascii_lowercase().as_str() {
        "read" | "grep" | "glob" => Some(FileAccess::Read),
        "write" | "edit" | "multiedit" | "notebookedit" => Some(FileAccess::Write),
        _ => None,
    }
}

/// Built-in protected globs and the label recorded when one matches.
/// `~` is expanded against the home directory at build time. `*` crosses
/// `/`, so `~/.ssh/**` covers every depth.
const BUILTIN: &[(&str, &str)] = &[
    ("~/.ssh/**", "SSH directory"),
    ("~/.gnupg/**", "GnuPG directory"),
    ("~/.aws/**", "AWS credentials directory"),
    ("~/.kube/config", "kubeconfig"),
    ("~/.docker/config.json", "Docker registry credentials"),
    ("~/.netrc", "netrc credentials"),
    ("~/.npmrc", "npm credentials"),
    ("~/.pypirc", "PyPI credentials"),
    ("~/.git-credentials", "git credential store"),
    ("~/.config/gh/hosts.yml", "GitHub CLI token"),
    ("**/.env", "dotenv file"),
    ("**/.env.*", "dotenv file"),
    ("**/id_rsa", "SSH private key"),
    ("**/id_ed25519", "SSH private key"),
    ("**/id_ecdsa", "SSH private key"),
    ("**/id_dsa", "SSH private key"),
    ("**/*.pem", "PEM key or certificate"),
    ("**/*.p12", "PKCS#12 keystore"),
    ("**/*.pfx", "PKCS#12 keystore"),
    ("**/*.keystore", "Java keystore"),
    ("**/.caro/config.toml", "caro config"),
];

/// Longest a user-supplied glob may be.
pub const MAX_GLOB_LEN: usize = 512;

/// Validate one user glob from `[safety] protected_paths`.
pub fn validate_user_glob(pattern: &str) -> Result<(), String> {
    if pattern.trim().is_empty() {
        return Err("empty protected_paths entry".into());
    }
    if pattern.len() > MAX_GLOB_LEN {
        return Err(format!(
            "protected_paths entry exceeds {MAX_GLOB_LEN} characters"
        ));
    }
    glob::Pattern::new(pattern)
        .map(|_| ())
        .map_err(|e| format!("invalid protected_paths glob '{pattern}': {e}"))
}

/// The compiled protected-path set.
#[derive(Debug, Clone)]
pub struct ProtectedPaths {
    globs: Vec<(glob::Pattern, String)>,
    /// Caro's own decision log; writes to it are denied.
    own_log: Option<PathBuf>,
    home: Option<PathBuf>,
}

/// One file-tool verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDecision {
    pub verdict: Verdict,
    /// Caro-authored label (never the path itself).
    pub label: String,
}

impl ProtectedPaths {
    /// Built-ins only, with caro's default log path.
    pub fn builtin() -> Self {
        Self::new(&[], super::log::default_log_path())
    }

    /// Built-ins plus user globs (invalid ones are skipped with a warning on
    /// stderr; they never widen or narrow the built-in set).
    pub fn new(user_globs: &[String], own_log: Option<PathBuf>) -> Self {
        let home = dirs::home_dir();
        let mut globs = Vec::with_capacity(BUILTIN.len() + user_globs.len());
        for (pat, label) in BUILTIN {
            push_glob(&mut globs, pat, label, home.as_deref());
        }
        for pat in user_globs {
            if let Err(e) = validate_user_glob(pat) {
                eprintln!("WARN: dropping protected_paths entry: {e}");
                continue;
            }
            push_glob(&mut globs, pat, "user protected path", home.as_deref());
        }
        // Caro's real config directory (XDG / platform specific), not only a
        // literal `.caro` folder: `~/.config/caro/**` on Linux.
        if let Some(cfg) = dirs::config_dir() {
            let pat = format!("{}/caro/**", cfg.to_string_lossy().trim_end_matches('/'));
            push_glob(&mut globs, &pat, "caro config directory", home.as_deref());
        }
        // A relative `--log` is relative to the hook process's cwd, which is
        // also the base the harness's relative paths resolve against.
        let process_cwd = std::env::current_dir().ok();
        Self {
            globs,
            own_log: own_log.map(|p| {
                PathBuf::from(normalize(
                    &p.to_string_lossy(),
                    None,
                    process_cwd.as_deref(),
                ))
            }),
            home,
        }
    }

    /// Decide for a `Glob`/`Grep` call: the optional search root, then the
    /// pattern joined onto it. The pattern is matched as a path string (so
    /// `~/.ssh/**` as a pattern hits the `~/.ssh/**` protection) and by its
    /// literal prefix (so `~/.ssh/id_*` hits it too).
    pub fn check_search(
        &self,
        access: FileAccess,
        root: Option<&str>,
        pattern: Option<&str>,
        cwd: Option<&str>,
    ) -> Option<PathDecision> {
        if let Some(r) = root.filter(|r| !r.is_empty()) {
            if let Some(d) = self.check(access, r, cwd) {
                return Some(d);
            }
        }
        let pattern = pattern.filter(|p| !p.is_empty())?;
        let base = root.filter(|r| !r.is_empty());
        let joined = |p: &str| match base {
            Some(b) if !p.starts_with('/') && !p.starts_with('~') => format!("{b}/{p}"),
            _ => p.to_string(),
        };
        if let Some(d) = self.check(access, &joined(pattern), cwd) {
            return Some(d);
        }
        let literal: String = pattern
            .chars()
            .take_while(|c| !matches!(c, '*' | '?' | '[' | '{'))
            .collect();
        let literal = literal.trim_end_matches('/');
        if literal.is_empty() || literal == pattern {
            return None;
        }
        self.check(access, &joined(literal), cwd)
    }

    /// Decide for one file-tool call. `cwd` resolves relative paths; `None`
    /// means a relative path is matched as given.
    pub fn check(
        &self,
        access: FileAccess,
        raw_path: &str,
        cwd: Option<&str>,
    ) -> Option<PathDecision> {
        let path = normalize(raw_path, self.home.as_deref(), cwd.map(Path::new));
        // A symlink in the repo (`credentials -> ~/.aws/credentials`) hides the
        // real target from a lexical match. One best-effort canonicalize of
        // the existing path closes that alias; a missing path stays lexical.
        if let Ok(real) = std::fs::canonicalize(&path) {
            let real = real.to_string_lossy().into_owned();
            if real != path {
                if let Some(d) = self.check_normalized(access, &real) {
                    return Some(d);
                }
            }
        }
        self.check_normalized(access, &path)
    }

    fn check_normalized(&self, access: FileAccess, path: &str) -> Option<PathDecision> {
        let path = path.to_string();
        if access == FileAccess::Write {
            if let Some(own) = &self.own_log {
                if Path::new(&path) == own.as_path() {
                    return Some(PathDecision {
                        verdict: Verdict::Deny,
                        label: "caro guard decision log".into(),
                    });
                }
            }
        }
        let opts = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };
        self.globs
            .iter()
            .find(|(g, _)| g.matches_with(&path, opts))
            .map(|(_, label)| PathDecision {
                verdict: Verdict::Ask,
                label: label.clone(),
            })
    }
}

/// Compile one glob. A directory glob (`dir/**`) also protects the directory
/// itself, so `Grep`/`Glob` rooted in `~/.ssh` are seen (Rampart closed the
/// same hole in 1.9.5).
fn push_glob(
    globs: &mut Vec<(glob::Pattern, String)>,
    pat: &str,
    label: &str,
    home: Option<&Path>,
) {
    let expanded = expand_tilde(pat, home);
    if let Ok(g) = glob::Pattern::new(&expanded) {
        globs.push((g, label.to_string()));
    }
    if let Some(dir) = expanded.strip_suffix("/**") {
        if let Ok(g) = glob::Pattern::new(dir) {
            globs.push((g, label.to_string()));
        }
    }
}

fn expand_tilde(pattern: &str, home: Option<&Path>) -> String {
    match (pattern.strip_prefix("~/"), home) {
        (Some(rest), Some(h)) => format!("{}/{rest}", h.to_string_lossy().trim_end_matches('/')),
        // No home dir: a `~` pattern can never match, keep it literal.
        _ => pattern.to_string(),
    }
}

/// Lexical normalization: expand `~`, join a relative path onto `cwd`,
/// drop `.` and resolve `..`. No filesystem access.
pub fn normalize(raw: &str, home: Option<&Path>, cwd: Option<&Path>) -> String {
    let expanded = expand_tilde(raw, home);
    let p = Path::new(&expanded);
    let joined: PathBuf = if p.is_absolute() {
        p.to_path_buf()
    } else if let Some(c) = cwd {
        c.join(p)
    } else {
        p.to_path_buf()
    };
    let mut out = PathBuf::new();
    for comp in joined.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> ProtectedPaths {
        ProtectedPaths::new(&[], Some(PathBuf::from("/data/caro/guard/decisions.jsonl")))
    }

    fn home() -> String {
        dirs::home_dir().unwrap().to_string_lossy().into_owned()
    }

    #[test]
    fn classifies_tools() {
        assert_eq!(file_access("Read"), Some(FileAccess::Read));
        assert_eq!(file_access("GLOB"), Some(FileAccess::Read));
        assert_eq!(file_access("Edit"), Some(FileAccess::Write));
        assert_eq!(file_access("NotebookEdit"), Some(FileAccess::Write));
        assert_eq!(file_access("Bash"), None);
        assert_eq!(file_access("WebFetch"), None);
    }

    #[test]
    fn normalizes_tilde_relative_and_dotdot() {
        let h = home();
        assert_eq!(
            normalize("~/.ssh/id_rsa", dirs::home_dir().as_deref(), None),
            format!("{h}/.ssh/id_rsa")
        );
        assert_eq!(
            normalize("src/../.env", None, Some(Path::new("/repo"))),
            "/repo/.env"
        );
        assert_eq!(normalize("./a/./b", None, Some(Path::new("/r"))), "/r/a/b");
    }

    #[test]
    fn ssh_key_read_asks_with_label_not_path() {
        let d = set()
            .check(FileAccess::Read, "~/.ssh/id_rsa", None)
            .unwrap();
        assert_eq!(d.verdict, Verdict::Ask);
        assert!(!d.label.contains("id_rsa"));
        // Depth under ~/.ssh is covered too, and so is the directory itself
        // (a Grep/Glob rooted there).
        assert!(set()
            .check(FileAccess::Read, "~/.ssh/keys/deploy", None)
            .is_some());
        assert!(set().check(FileAccess::Read, "~/.ssh", None).is_some());
        assert!(set().check(FileAccess::Read, "~/.sshx", None).is_none());
    }

    #[test]
    fn dotenv_anywhere_asks_and_dotenv_example_does_too() {
        assert_eq!(
            set()
                .check(FileAccess::Read, ".env", Some("/repo"))
                .unwrap()
                .verdict,
            Verdict::Ask
        );
        assert!(set()
            .check(FileAccess::Write, "/repo/apps/web/.env.local", None)
            .is_some());
    }

    #[test]
    fn routine_source_paths_have_no_opinion() {
        for p in [
            "src/main.rs",
            "/repo/README.md",
            "Cargo.toml",
            "/etc/hosts",
            "env.rs",
            "environment.toml",
        ] {
            assert!(
                set().check(FileAccess::Read, p, Some("/repo")).is_none(),
                "{p}"
            );
            assert!(
                set().check(FileAccess::Write, p, Some("/repo")).is_none(),
                "{p}"
            );
        }
    }

    #[test]
    fn writing_own_log_is_denied_reading_is_not() {
        let d = set()
            .check(
                FileAccess::Write,
                "/data/caro/guard/../guard/decisions.jsonl",
                None,
            )
            .unwrap();
        assert_eq!(d.verdict, Verdict::Deny);
        assert!(set()
            .check(FileAccess::Read, "/data/caro/guard/decisions.jsonl", None)
            .is_none());
    }

    #[test]
    fn glob_pattern_is_checked_not_only_the_root() {
        let s = set();
        assert!(s
            .check_search(FileAccess::Read, None, Some("~/.ssh/**"), None)
            .is_some());
        assert!(s
            .check_search(FileAccess::Read, None, Some("~/.ssh/id_*"), None)
            .is_some());
        assert!(s
            .check_search(FileAccess::Read, Some("/repo"), Some("**/.env"), None)
            .is_some());
        assert!(s
            .check_search(
                FileAccess::Read,
                Some(&format!("{}/.aws", home())),
                Some("*"),
                None
            )
            .is_some());
        assert!(s
            .check_search(FileAccess::Read, Some("/repo"), Some("**/*.rs"), None)
            .is_none());
        assert!(s
            .check_search(FileAccess::Read, None, None, Some("/repo"))
            .is_none());
    }

    #[test]
    fn caro_config_dir_and_relative_log_are_protected() {
        let cfg = dirs::config_dir().unwrap().join("caro").join("config.toml");
        assert!(set()
            .check(FileAccess::Read, &cfg.to_string_lossy(), None)
            .is_some());
        let rel = ProtectedPaths::new(&[], Some(PathBuf::from("decisions.jsonl")));
        let cwd = std::env::current_dir().unwrap();
        let d = rel
            .check(
                FileAccess::Write,
                "decisions.jsonl",
                Some(&cwd.to_string_lossy()),
            )
            .unwrap();
        assert_eq!(d.verdict, Verdict::Deny);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_alias_to_a_protected_path_is_seen() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("secret.pem");
        std::fs::write(&target, "x").unwrap();
        let link = dir.path().join("harmless.txt");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(set()
            .check(FileAccess::Read, &link.to_string_lossy(), None)
            .is_some());
    }

    #[test]
    fn user_globs_are_additive_and_validated() {
        let s = ProtectedPaths::new(&["**/secrets/**".into(), "".into(), "[".into()], None);
        assert!(s
            .check(FileAccess::Read, "/repo/secrets/token", None)
            .is_some());
        assert!(
            s.check(FileAccess::Read, "~/.ssh/id_rsa", None).is_some(),
            "built-ins stay"
        );
        assert!(validate_user_glob("[").is_err());
        assert!(validate_user_glob(&"a".repeat(MAX_GLOB_LEN + 1)).is_err());
    }
}
