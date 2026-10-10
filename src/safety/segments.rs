//! Statement splitting and wrapper unwrapping for the safety scanners (#1054).
//!
//! The pattern scanners look at whole strings. Two shapes hid a dangerous
//! command from them:
//!
//! - a statement after a separator (`git status; chmod -R 777 ~`), when the
//!   pattern needed the command at a statement start;
//! - a payload inside an exec wrapper (`bash -c 'rm -rf /'`, `eval 'rm -rf /'`),
//!   where the quote-parity check treats the payload as an inert string.
//!
//! [`scan_views`] returns extra strings for the scanners to check: each
//! statement, each statement with its prefix wrappers removed (`sudo`, `env`,
//! `timeout`, …), and each exec-wrapper payload, recursively to
//! [`MAX_DEPTH`]. The original command is NOT included; the caller scans it
//! too.
//!
//! What is never unwrapped: arguments of `echo`, `printf` or any other
//! command that only prints or searches text. A dangerous string there is
//! data, not code (`tests/safety_validator_contract.rs` pins this).
//!
//! This is a scanner aid, not a shell parser. It handles `'…'`, `"…"`,
//! backslash escapes, `$(…)` and backticks well enough to find statement
//! boundaries. Anything it cannot tokenize is left to the whole-string scan.

/// Maximum wrapper nesting that is unwrapped (`bash -c "sh -c 'eval …'"`).
pub const MAX_DEPTH: usize = 3;

/// Upper bound on the number of views, so a hostile input cannot make the
/// scanners do unbounded work.
const MAX_VIEWS: usize = 64;

/// Shells whose `-c` argument is code.
const SHELLS: &[&str] = &["sh", "bash", "zsh", "dash", "ksh", "mksh", "ash", "fish"];

/// Split a command line into top-level statements on `;`, `&&`, `||`, `|`,
/// `&` and newlines. Separators inside quotes, `$(…)` or backticks are not
/// split points. Empty statements are dropped; each is trimmed.
pub fn split_statements(command: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = command.chars().peekable();
    let (mut single, mut double, mut backtick) = (false, false, false);
    let mut paren_depth = 0usize;

    while let Some(c) = chars.next() {
        if single {
            cur.push(c);
            if c == '\'' {
                single = false;
            }
            continue;
        }
        match c {
            '\\' => {
                cur.push(c);
                if let Some(n) = chars.next() {
                    cur.push(n);
                }
            }
            '\'' if !double => {
                single = true;
                cur.push(c);
            }
            '"' => {
                double = !double;
                cur.push(c);
            }
            '`' => {
                backtick = !backtick;
                cur.push(c);
            }
            '$' if chars.peek() == Some(&'(') => {
                cur.push(c);
                cur.push(chars.next().unwrap_or('('));
                paren_depth += 1;
            }
            ')' if paren_depth > 0 => {
                paren_depth -= 1;
                cur.push(c);
            }
            ';' | '&' | '|' | '\n' if !double && !backtick && paren_depth == 0 => {
                // `&&` / `||` are one separator; `|&` too.
                if matches!(chars.peek(), Some('&') | Some('|')) {
                    chars.next();
                }
                push_trimmed(&mut out, &cur);
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    push_trimmed(&mut out, &cur);
    out
}

fn push_trimmed(out: &mut Vec<String>, s: &str) {
    let t = s.trim();
    if !t.is_empty() {
        out.push(t.to_string());
    }
}

fn base_name(tok: &str) -> &str {
    tok.rsplit('/').next().unwrap_or(tok)
}

/// Remove prefix wrappers that run their arguments as a command:
/// `sudo`/`doas` (with flags), `env` (with flags and `VAR=…`), `timeout`,
/// `nice`, `nohup`, `exec`, `command`, `builtin`, `xargs` (with flags), and
/// leading `VAR=value` assignments. Returns the remaining argv.
fn strip_prefixes(mut argv: &[String]) -> &[String] {
    loop {
        let Some(first) = argv.first() else {
            return argv;
        };
        let word = base_name(first);
        let rest = &argv[1..];
        let skip_flags = |args: &[String]| args.iter().take_while(|a| a.starts_with('-')).count();
        argv = match word {
            _ if is_assignment(first) => rest,
            "sudo" | "doas" => {
                // sudo flags that take a value: -u, -g, -C, -D, -h, -p, -r, -t, -U.
                let mut i = 0;
                while i < rest.len() && rest[i].starts_with('-') {
                    let takes_value = matches!(
                        rest[i].as_str(),
                        "-u" | "-g" | "-C" | "-D" | "-h" | "-p" | "-r" | "-t" | "-U"
                    );
                    i += if takes_value { 2 } else { 1 };
                }
                &rest[i.min(rest.len())..]
            }
            "env" => {
                let mut i = skip_flags(rest);
                while i < rest.len() && is_assignment(&rest[i]) {
                    i += 1;
                }
                &rest[i..]
            }
            "timeout" => {
                // timeout [flags] DURATION cmd…
                let i = skip_flags(rest);
                &rest[(i + 1).min(rest.len())..]
            }
            "nice" => {
                let mut i = 0;
                while i < rest.len() && rest[i].starts_with('-') {
                    i += if rest[i] == "-n" { 2 } else { 1 };
                }
                &rest[i.min(rest.len())..]
            }
            "nohup" | "exec" | "command" | "builtin" | "time" => rest,
            "xargs" => &rest[skip_flags(rest)..],
            _ => return argv,
        };
    }
}

fn is_assignment(tok: &str) -> bool {
    match tok.split_once('=') {
        Some((name, _)) => {
            !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !name.starts_with(|c: char| c.is_ascii_digit())
        }
        None => false,
    }
}

/// The code an exec wrapper runs: the `-c` argument of a shell, or the
/// arguments of `eval` joined by spaces. `None` for any other command.
fn wrapper_payload(argv: &[String]) -> Option<String> {
    let word = base_name(argv.first()?);
    if word == "eval" {
        let joined = argv[1..].join(" ");
        return (!joined.trim().is_empty()).then_some(joined);
    }
    if SHELLS.contains(&word) {
        // Find the first option cluster that contains `c` (`-c`, `-lc`,
        // `-ec`); the payload is the next non-option argument.
        let mut iter = argv[1..].iter();
        while let Some(arg) = iter.next() {
            if arg == "--" || !arg.starts_with('-') {
                return None;
            }
            if !arg.starts_with("--") && arg[1..].contains('c') {
                return iter.find(|a| !a.starts_with('-')).cloned();
            }
        }
    }
    None
}

/// Extra strings for the scanners: statements, prefix-stripped statements
/// and wrapper payloads (recursively). Excludes the original `command` and
/// removes duplicates.
pub fn scan_views(command: &str) -> Vec<String> {
    let mut views: Vec<String> = Vec::new();
    collect(command, 0, &mut views);
    views.retain(|v| v != command);
    views.dedup();
    views
}

fn collect(command: &str, depth: usize, views: &mut Vec<String>) {
    for stmt in split_statements(command) {
        if views.len() >= MAX_VIEWS {
            return;
        }
        if !views.contains(&stmt) {
            views.push(stmt.clone());
        }
        let Ok(argv) = shell_words::split(&stmt) else {
            continue;
        };
        let inner = strip_prefixes(&argv);
        if inner.is_empty() {
            continue;
        }
        if inner.len() != argv.len() {
            // Re-quote so the stripped view keeps its argument boundaries.
            let stripped = shell_words::join(inner);
            if !views.contains(&stripped) {
                views.push(stripped);
            }
        }
        if depth < MAX_DEPTH {
            if let Some(payload) = wrapper_payload(inner) {
                if !views.contains(&payload) {
                    views.push(payload.clone());
                }
                collect(&payload, depth + 1, views);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_all_separators() {
        assert_eq!(
            split_statements("a; b && c || d | e & f\ng"),
            vec!["a", "b", "c", "d", "e", "f", "g"]
        );
    }

    #[test]
    fn does_not_split_inside_quotes_or_substitutions() {
        assert_eq!(split_statements("echo 'a; b'"), vec!["echo 'a; b'"]);
        assert_eq!(split_statements("echo \"a && b\""), vec!["echo \"a && b\""]);
        assert_eq!(split_statements("x=$(a | b); y"), vec!["x=$(a | b)", "y"]);
        assert_eq!(split_statements("echo `a; b`"), vec!["echo `a; b`"]);
        assert_eq!(
            split_statements("find . -exec rm {} \\; ; ls"),
            vec!["find . -exec rm {} \\;", "ls"]
        );
    }

    #[test]
    fn unwraps_shell_c_and_eval() {
        assert!(scan_views("bash -c 'rm -rf /'").contains(&"rm -rf /".to_string()));
        assert!(scan_views("sh -lc \"rm -rf ~\"").contains(&"rm -rf ~".to_string()));
        assert!(scan_views("eval 'rm -rf /'").contains(&"rm -rf /".to_string()));
        assert!(scan_views("sudo -u root bash -c 'rm -rf /'").contains(&"rm -rf /".to_string()));
        assert!(scan_views("env A=1 timeout 5 nice -n 5 bash -c 'rm -rf /'")
            .contains(&"rm -rf /".to_string()));
    }

    #[test]
    fn strips_prefixes_into_a_view() {
        assert!(scan_views("sudo find / -delete").contains(&"find / -delete".to_string()));
    }

    #[test]
    fn nested_wrappers_stop_at_max_depth() {
        let views = scan_views("bash -c \"sh -c 'eval rm -rf /'\"");
        assert!(views.contains(&"rm -rf /".to_string()));
    }

    #[test]
    fn never_unwraps_print_or_search_commands() {
        for cmd in [
            "echo 'rm -rf /'",
            "printf '%s' 'bash -c \"rm -rf /\"'",
            "echo \"bash -c 'rm -rf /'\"",
            "grep -r 'eval rm -rf /' src",
        ] {
            let views = scan_views(cmd);
            assert!(
                !views.iter().any(|v| v.starts_with("rm ")),
                "{cmd:?} exposed a payload: {views:?}"
            );
        }
    }

    #[test]
    fn view_count_is_bounded() {
        let hostile = "a;".repeat(1000);
        assert!(scan_views(&hostile).len() <= MAX_VIEWS);
    }
}
