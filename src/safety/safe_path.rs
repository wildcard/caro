//! Path-aware safe-deletion classifier for the allowlist override (caro-qknc).
//!
//! The safety validator never lets a user allowlist bypass a Critical command
//! ([`SafetyValidator::validate_command`](super::SafetyValidator::validate_command)).
//! This module carves a *narrow, fail-closed* exception: a recursive `rm` whose
//! every target provably resolves **strictly inside** a safe root (temp dirs, or
//! the project working directory) may be allowlisted, while `rm -rf /`,
//! `rm -rf /etc`, traversal/symlink escapes, and anything ambiguous stay blocked.
//!
//! Design + threat model: `.claude/beta-testing/threats/path-aware-allowlist.md`.
//!
//! Scope caveat: the safety guarantee is evaluated at *check* time, not *use*
//! time. A local attacker who plants a symlink into a world-writable safe root
//! (e.g. `/tmp`) between validation and the `rm` could still redirect a
//! not-yet-existing target (a check-then-use / TOCTOU race). The classifier
//! closes the far more reachable *static* escapes (traversal, symlinked dirs,
//! glob-masked symlinks); the residual race is inherent to validating a command
//! string rather than performing the deletion atomically.

use std::path::{Component, Path, PathBuf};

/// Shell metacharacters that make a command too complex to prove safe. Their
/// presence makes the command ineligible for the override (fail-closed) — the
/// exception only ever applies to dead-simple `rm -rf <plain paths>`.
/// Glob chars (`* ? [ ]`) are intentionally *allowed*: they are normalized as
/// literal path segments, so `/tmp/*` stays inside `/tmp` while `/tmp/../*`
/// normalizes outside it and is rejected.
const UNSAFE_META: &[char] = &[
    '$', '`', '"', '\'', '\\', ';', '|', '&', '>', '<', '(', ')', '{', '}', '~', '\n', '\r', '!',
];

/// Directories that must never be treated as a project safe-root even when they
/// are the current working directory (e.g. running caro from `/` or `$HOME`).
const SENSITIVE_ROOTS: &[&str] = &[
    "/", "/etc", "/usr", "/var", "/bin", "/sbin", "/lib", "/lib64", "/boot", "/opt", "/root",
    "/home", "/Users", "/System", "/Library", "/private", "/dev", "/proc", "/sys", "/tmp",
    "/var/tmp",
];

/// Build the set of safe roots: temp dirs always, plus `cwd` when it is a
/// legitimate (non-sensitive) project directory.
pub fn safe_roots(cwd: &Path) -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/tmp"),
        PathBuf::from("/var/tmp"),
        std::env::temp_dir(),
    ];
    if is_acceptable_project_cwd(cwd) {
        roots.push(cwd.to_path_buf());
    }
    roots
}

fn is_acceptable_project_cwd(cwd: &Path) -> bool {
    if !cwd.is_absolute() {
        return false;
    }
    let norm = lexical_normalize(cwd);
    if SENSITIVE_ROOTS.iter().any(|s| Path::new(s) == norm) {
        return false;
    }
    // Never accept $HOME itself as a project root (its subpaths like ~/.ssh are
    // sensitive); a project *under* $HOME is fine.
    if let Some(home) = std::env::var_os("HOME") {
        if lexical_normalize(Path::new(&home)) == norm {
            return false;
        }
    }
    true
}

/// Returns `true` iff `command` is a recursive `rm` whose every target resolves
/// strictly inside one of `roots`. Fail-closed on any ambiguity.
pub fn is_scoped_safe_deletion(command: &str, roots: &[PathBuf], cwd: &Path) -> bool {
    // 1. Reject any shell complexity (quoting, substitution, chaining, ~, env).
    if command.chars().any(|c| UNSAFE_META.contains(&c)) {
        return false;
    }

    // 2. Whitespace tokenization is sufficient now that metachars are excluded.
    let tokens: Vec<&str> = command.split_whitespace().collect();
    let Some((&cmd0, rest)) = tokens.split_first() else {
        return false;
    };

    // 3. Only `rm` deletions are eligible (not mkfs/dd/etc. — those are Critical
    //    for reasons unrelated to a path scope).
    if cmd0 != "rm" && !cmd0.ends_with("/rm") {
        return false;
    }

    // 4. Collect targets; require a recursive flag.
    let mut recursive = false;
    let mut targets = Vec::new();
    for &tok in rest {
        if tok == "--" {
            continue;
        }
        if tok.starts_with('-') {
            if tok.contains('r') || tok.contains('R') {
                recursive = true;
            }
            continue;
        }
        targets.push(tok);
    }
    if !recursive || targets.is_empty() {
        return false;
    }

    // 5. Every target must resolve strictly inside a safe root.
    targets.iter().all(|t| target_in_safe_root(t, roots, cwd))
}

fn target_in_safe_root(target: &str, roots: &[PathBuf], cwd: &Path) -> bool {
    // A glob may appear ONLY in the final path component. A glob in a
    // non-final component (e.g. `/tmp/*/x`) is fail-closed rejected: the shell
    // expands it and `rm` follows the expanded directory *through* a symlink,
    // but the classifier only sees the literal (non-existent) pattern, so the
    // symlink canonicalization below never runs on it — a real escape
    // (`/tmp/link -> /etc` ⇒ `/tmp/*/x` deletes `/etc/x`). A *leaf* glob is
    // safe because `rm` unlinks a matched symlink operand rather than
    // recursing through it. See aegis review, caro-qknc.
    if !glob_position_ok(target) {
        return false;
    }

    let raw = Path::new(target);
    let joined = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        cwd.join(raw)
    };
    let norm = lexical_normalize(&joined);

    // Lexical containment: a *proper* subpath of some safe root (never the root
    // itself — deleting all of /tmp or the whole project dir is not "scoped").
    let lexically_ok = roots.iter().any(|root| {
        let root_n = lexical_normalize(root);
        norm != root_n && norm.starts_with(&root_n)
    });
    if !lexically_ok {
        return false;
    }

    // Hybrid (D2): if the path or its nearest existing ancestor exists,
    // canonicalize to catch symlink escapes (e.g. /tmp/link -> /etc).
    canonical_still_safe(&norm, roots)
}

/// Glob metacharacters that the shell expands (and whose expansion can follow
/// symlinks for directory components).
const GLOB_CHARS: &[char] = &['*', '?', '[', ']'];

/// A glob is permitted ONLY in the final path component and never with a
/// trailing slash (which forces directory semantics on a symlinked match).
/// Everything else with a glob is rejected fail-closed.
fn glob_position_ok(target: &str) -> bool {
    if !target.contains(GLOB_CHARS) {
        return true; // concrete path — symlink handling is done via canonicalize
    }
    if target.ends_with('/') {
        return false; // glob + trailing slash → directory semantics on a match
    }
    let comps: Vec<Component> = Path::new(target).components().collect();
    if comps.is_empty() {
        return false;
    }
    // No glob char may appear in any component before the last.
    comps[..comps.len() - 1].iter().all(|c| match c {
        Component::Normal(seg) => !seg.to_string_lossy().contains(GLOB_CHARS),
        _ => true,
    })
}

/// Resolve `.` and `..` lexically without touching the filesystem. Glob chars
/// are preserved as literal segments.
fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::RootDir => out.push("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                // Pop a normal segment; never pop above root.
                if !out.pop() {
                    out.push("..");
                }
            }
            Component::Normal(seg) => out.push(seg),
            Component::Prefix(p) => out.push(p.as_os_str()),
        }
    }
    if out.as_os_str().is_empty() {
        out.push("/");
    }
    out
}

/// Resolve symlinks by canonicalizing the nearest existing ancestor of `norm`,
/// re-appending the not-yet-existing tail, and requiring the resulting real path
/// to stay a strict subpath of some safe root (each root canonicalized when it
/// exists, else taken lexically so a not-yet-created project dir still works).
/// Fail-closed if an existing path cannot be canonicalized. This is what catches
/// a symlink escape such as `/tmp/link -> /etc`.
fn canonical_still_safe(norm: &Path, roots: &[PathBuf]) -> bool {
    // Split `norm` into (nearest existing ancestor, non-existing tail).
    let mut anc = norm.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let real_anc = loop {
        if anc.exists() {
            match anc.canonicalize() {
                Ok(canon) => break canon,
                Err(_) => return false, // exists but un-canonicalizable -> fail closed
            }
        }
        match anc.file_name() {
            Some(name) => {
                tail.push(name.to_os_string());
                match anc.parent() {
                    Some(parent) => anc = parent.to_path_buf(),
                    None => break PathBuf::from("/"),
                }
            }
            None => break anc.clone(), // reached root
        }
    };

    // Reconstruct the real target path.
    let mut realpath = real_anc;
    for seg in tail.iter().rev() {
        realpath.push(seg);
    }

    // Strict-subpath containment against canonical-or-lexical roots.
    roots.iter().any(|root| {
        let root_real = root
            .canonicalize()
            .unwrap_or_else(|_| lexical_normalize(root));
        realpath != root_real && realpath.starts_with(&root_real)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots() -> Vec<PathBuf> {
        vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")]
    }
    fn cwd() -> PathBuf {
        PathBuf::from("/home/dev/project")
    }

    // ---- ALLOWED: scoped deletions inside safe roots --------------------
    #[test]
    fn allows_temp_subpath() {
        assert!(is_scoped_safe_deletion(
            "rm -rf /tmp/myapp_123",
            &roots(),
            &cwd()
        ));
    }

    #[test]
    fn allows_flag_order_and_variants() {
        assert!(is_scoped_safe_deletion("rm -fr /tmp/x", &roots(), &cwd()));
        assert!(is_scoped_safe_deletion(
            "rm -r -f /tmp/build",
            &roots(),
            &cwd()
        ));
        assert!(is_scoped_safe_deletion(
            "rm -rf /tmp/build/",
            &roots(),
            &cwd()
        ));
    }

    #[test]
    fn allows_glob_inside_temp() {
        // glob normalizes as a literal segment under /tmp -> safe
        assert!(is_scoped_safe_deletion(
            "rm -rf /tmp/myapp_*",
            &roots(),
            &cwd()
        ));
    }

    #[test]
    fn allows_project_relative_subpath() {
        // Use the real current dir: cwd always exists and canonicalizes (the
        // hybrid check resolves symlinks on the nearest existing ancestor).
        let cwd = std::env::current_dir().unwrap();
        let r = safe_roots(&cwd);
        assert!(is_scoped_safe_deletion("rm -rf ./build", &r, &cwd));
        assert!(is_scoped_safe_deletion("rm -rf target", &r, &cwd));
    }

    // ---- BLOCKED: T1 system/root paths ----------------------------------
    #[test]
    fn blocks_root_and_system_paths() {
        for cmd in [
            "rm -rf /",
            "rm -rf /etc",
            "rm -rf /usr",
            "rm -rf /var",
            "rm -rf /bin/sh",
        ] {
            assert!(!is_scoped_safe_deletion(cmd, &roots(), &cwd()), "{cmd}");
        }
    }

    // ---- T3 traversal escape --------------------------------------------
    #[test]
    fn blocks_traversal_escape() {
        assert!(!is_scoped_safe_deletion(
            "rm -rf /tmp/../etc",
            &roots(),
            &cwd()
        ));
        assert!(!is_scoped_safe_deletion(
            "rm -rf /tmp/a/../../usr",
            &roots(),
            &cwd()
        ));
    }

    // ---- T5 glob escape -------------------------------------------------
    #[test]
    fn blocks_glob_escape() {
        assert!(!is_scoped_safe_deletion(
            "rm -rf /tmp/../*",
            &roots(),
            &cwd()
        ));
    }

    // ---- T4∘T5 composition: non-final glob masks a symlinked dir --------
    // Regression for the aegis-confirmed bypass: `/tmp/*/x` validates the
    // literal (non-existent) pattern, so canonicalization never resolves the
    // symlink the shell expands `*` into.
    #[test]
    fn blocks_non_final_glob() {
        for cmd in [
            "rm -rf /tmp/*/x",
            "rm -rf /tmp/a*/x",
            "rm -rf /tmp/*/passwd",
            "rm -rf /tmp/*/",  // trailing slash on glob
            "rm -rf /tmp/?/x", // ? in non-final component
        ] {
            assert!(!is_scoped_safe_deletion(cmd, &roots(), &cwd()), "{cmd}");
        }
    }

    // Real-filesystem proof: with a symlink `<tmp>/link -> /etc`, the literal
    // `<tmp>/link/x` is (correctly) rejected AND the glob form `<tmp>/*/x`
    // must be too — both denote the same escaping deletion.
    #[test]
    fn blocks_glob_over_symlinked_dir_on_real_fs() {
        let base = std::env::temp_dir().join(format!("caro_qknc_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc", base.join("link")).unwrap();
        let roots = vec![base.clone()];
        let cwd = std::env::current_dir().unwrap();

        let literal = format!("rm -rf {}/link/x", base.display());
        let globbed = format!("rm -rf {}/*/x", base.display());
        assert!(
            !is_scoped_safe_deletion(&literal, &roots, &cwd),
            "literal symlink escape"
        );
        assert!(
            !is_scoped_safe_deletion(&globbed, &roots, &cwd),
            "glob-masked symlink escape"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn leaf_glob_still_allowed() {
        // The intended feature: a glob only in the final component.
        assert!(is_scoped_safe_deletion(
            "rm -rf /tmp/myapp_*",
            &roots(),
            &cwd()
        ));
        assert!(is_scoped_safe_deletion(
            "rm -rf /tmp/build_?",
            &roots(),
            &cwd()
        ));
    }

    // ---- the root itself is not a "scoped" subpath ----------------------
    #[test]
    fn blocks_deleting_the_safe_root_itself() {
        assert!(!is_scoped_safe_deletion("rm -rf /tmp", &roots(), &cwd()));
        assert!(!is_scoped_safe_deletion("rm -rf /tmp/", &roots(), &cwd()));
    }

    // ---- T6 mixed targets: any unsafe -> reject whole command -----------
    #[test]
    fn blocks_when_any_target_unsafe() {
        assert!(!is_scoped_safe_deletion(
            "rm -rf /tmp/ok /etc",
            &roots(),
            &cwd()
        ));
    }

    // ---- T7 indirection / shell complexity -> fail closed ---------------
    #[test]
    fn blocks_shell_complexity() {
        for cmd in [
            "rm -rf $X",
            "rm -rf $HOME/x",
            "rm -rf ~/x",
            "rm -rf \"$(echo /tmp/x)\"",
            "rm -rf /tmp/x; rm -rf /etc",
            "rm -rf /tmp/x && rm -rf /",
            "rm -rf /tmp/x | tee",
        ] {
            assert!(!is_scoped_safe_deletion(cmd, &roots(), &cwd()), "{cmd}");
        }
    }

    // ---- non-rm / non-recursive not eligible ----------------------------
    #[test]
    fn blocks_non_rm_and_non_recursive() {
        assert!(!is_scoped_safe_deletion("mkfs /tmp/x", &roots(), &cwd()));
        assert!(!is_scoped_safe_deletion(
            "dd if=/dev/zero of=/tmp/x",
            &roots(),
            &cwd()
        ));
        assert!(!is_scoped_safe_deletion("rm -f /tmp/x", &roots(), &cwd())); // not recursive
        assert!(!is_scoped_safe_deletion("rm -rf", &roots(), &cwd())); // no target
    }

    // ---- project cwd guard: never accept sensitive cwd as a root --------
    #[test]
    fn rejects_sensitive_cwd_as_project_root() {
        for bad in ["/", "/etc", "/home", "/Users"] {
            let r = safe_roots(Path::new(bad));
            // cwd must NOT have been added, so a relative target there is unsafe
            assert!(
                !is_scoped_safe_deletion("rm -rf ./x", &r, Path::new(bad)),
                "{bad}"
            );
        }
    }

    #[test]
    fn lexical_normalize_resolves_dotdot() {
        assert_eq!(
            lexical_normalize(Path::new("/tmp/a/../b")),
            PathBuf::from("/tmp/b")
        );
        assert_eq!(
            lexical_normalize(Path::new("/tmp/../etc")),
            PathBuf::from("/etc")
        );
    }
}
