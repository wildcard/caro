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
//! data, not code (`tests/safety_validator_contract.rs` pins this). A
//! statement whose command is `echo` or `printf` is not a view either, so
//! `bash -c 'echo rm -rf /'` exposes no `rm -rf /` string.
//!
//! There is no cap on the number of views. A cap would let a long run of
//! harmless statements push a dangerous one out of the scan. The work is
//! bounded by the validator's `max_command_length` and by [`MAX_DEPTH`].
//!
//! This is a scanner aid, not a shell parser. It handles `'…'`, `"…"`,
//! backslash escapes, `$(…)` and backticks well enough to find statement
//! boundaries. Anything it cannot tokenize is left to the whole-string scan.

/// Maximum wrapper nesting that is unwrapped (`bash -c "sh -c 'eval …'"`).
pub const MAX_DEPTH: usize = 3;

/// Shells whose `-c` argument is code.
const SHELLS: &[&str] = &["sh", "bash", "zsh", "dash", "ksh", "mksh", "ash", "fish"];

/// Commands whose arguments are only printed. Their statements are not views.
const PRINT_ONLY: &[&str] = &["echo", "printf"];

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

/// Count the leading options in `args`. An option in `takes_value` also
/// consumes the next argument (`-u FOO`). A `--` ends the options and is
/// counted.
fn skip_options(args: &[String], takes_value: &[&str]) -> usize {
    let mut i = 0;
    while i < args.len() && args[i].starts_with('-') {
        if args[i] == "--" {
            return i + 1;
        }
        i += if takes_value.contains(&args[i].as_str()) {
            2
        } else {
            1
        };
    }
    i.min(args.len())
}

/// Remove prefix wrappers that run their arguments as a command:
/// `sudo`/`doas` (with flags), `env` (with flags and `VAR=…`), `timeout`,
/// `nice`, `nohup`, `exec`, `command`, `builtin`, `xargs` (with flags), and
/// leading `VAR=value` assignments. Returns the remaining argv.
fn strip_prefixes(mut argv: &[String]) -> &[String] {
    while let Some(rest) = strip_one(argv) {
        argv = rest;
    }
    argv
}

/// Remove one prefix wrapper (with its options) from the front of `argv`.
/// `None` when `argv` does not start with one.
fn strip_one(argv: &[String]) -> Option<&[String]> {
    let first = argv.first()?;
    let word = base_name(first);
    let rest = &argv[1..];
    Some(match word {
        _ if is_assignment(first) => rest,
        "sudo" | "doas" => {
            let i = skip_options(
                rest,
                &["-u", "-g", "-C", "-D", "-h", "-p", "-r", "-t", "-U"],
            );
            &rest[i..]
        }
        "env" => {
            // `-S` carries a whole command line; `env_split_payload` scans it.
            let mut i = skip_options(
                rest,
                &["-u", "--unset", "-C", "--chdir", "-S", "--split-string"],
            );
            while i < rest.len() && is_assignment(&rest[i]) {
                i += 1;
            }
            &rest[i..]
        }
        "timeout" => {
            // timeout [options] DURATION cmd…
            let i = skip_options(rest, &["-s", "--signal", "-k", "--kill-after"]);
            &rest[(i + 1).min(rest.len())..]
        }
        "nice" => &rest[skip_options(rest, &["-n", "--adjustment"])..],
        "nohup" | "exec" | "command" | "builtin" | "time" | "busybox" => rest,
        "xargs" => {
            let i = skip_options(
                rest,
                &[
                    "-I",
                    "-n",
                    "-P",
                    "-L",
                    "-s",
                    "-d",
                    "-E",
                    "-a",
                    "--max-args",
                    "--max-procs",
                    "--max-lines",
                    "--max-chars",
                    "--delimiter",
                    "--arg-file",
                ],
            );
            &rest[i..]
        }
        _ => return None,
    })
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
        return shell_c_payload(&argv[1..]);
    }
    None
}

/// The script of `sh -c`: the first non-option argument, when an option
/// cluster contains `c` (`-c`, `-lc`, `-ec`). Like bash and dash, options
/// may come before or after `-c`, and `-o`/`+o`/`-O`/`+O`, `--rcfile` and
/// `--init-file` take a value. `--` or `-` ends the options.
fn shell_c_payload(args: &[String]) -> Option<String> {
    let mut has_c = false;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" || arg == "-" {
            i += 1;
            break;
        }
        if let Some(long) = arg.strip_prefix("--") {
            i += if matches!(long, "rcfile" | "init-file") {
                2
            } else {
                1
            };
            continue;
        }
        let Some(flags) = arg.strip_prefix('-').or_else(|| arg.strip_prefix('+')) else {
            break;
        };
        if arg.starts_with('-') && flags.contains('c') {
            has_c = true;
        }
        i += if flags.ends_with(['o', 'O']) { 2 } else { 1 };
    }
    if has_c {
        args.get(i).cloned()
    } else {
        None
    }
}

/// The command line in `env -S STRING` / `env --split-string=STRING`, with
/// any arguments after it. `env` counts only where it is the command, after
/// zero or more prefix wrappers (`sudo env -S …`, not `true env -S …`).
fn env_split_payload(mut argv: &[String]) -> Option<String> {
    while base_name(argv.first()?) != "env" {
        argv = strip_one(argv)?;
    }
    let mut iter = argv[1..].iter();
    while let Some(arg) = iter.next() {
        let value = match arg.as_str() {
            "-S" | "--split-string" => iter.next()?.clone(),
            "-u" | "--unset" | "-C" | "--chdir" => {
                iter.next();
                continue;
            }
            a if a.starts_with("--split-string=") => a["--split-string=".len()..].to_string(),
            a if a.starts_with("-S") => a[2..].to_string(),
            a if a.starts_with('-') => continue,
            _ => return None,
        };
        let rest: Vec<&str> = iter.map(String::as_str).collect();
        return Some(if rest.is_empty() {
            value
        } else {
            format!("{value} {}", join_view(&rest))
        });
    }
    None
}

/// The payload of the first exec wrapper anywhere after `argv[0]`. A
/// fallback for prefix wrappers with options that [`strip_prefixes`] does
/// not know (`timeout --preserve-status -s TERM 5 bash -c …`).
fn payload_after_prefix(argv: &[String]) -> Option<String> {
    argv.iter()
        .enumerate()
        .skip(1)
        .filter(|(_, a)| {
            let w = base_name(a);
            w == "eval" || SHELLS.contains(&w)
        })
        .find_map(|(i, _)| wrapper_payload(&argv[i..]))
}

/// Join tokens back into a command line for a view. Only a token that would
/// lose its word boundaries (blanks, separators, quotes) is quoted. `~`,
/// `$HOME`, `*` and `{}` stay bare, so the scanners still read `~` as home:
/// `shell_words::join` would write `'~'`, which is a literal name.
fn join_view<S: AsRef<str>>(tokens: &[S]) -> String {
    tokens
        .iter()
        .map(|t| {
            let t = t.as_ref();
            let needs_quotes = t.is_empty()
                || t.chars()
                    .any(|c| c.is_whitespace() || ";&|<>()'\"`\\".contains(c));
            if needs_quotes {
                shell_words::quote(t).into_owned()
            } else {
                t.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extra strings for the scanners: statements, prefix-stripped statements
/// and wrapper payloads (recursively). Excludes the original `command` and
/// removes duplicates.
pub fn scan_views(command: &str) -> Vec<String> {
    let mut views = Views::default();
    collect(command, 0, &mut views);
    views.list.retain(|v| v != command);
    views.list
}

/// Views in first-seen order, without duplicates.
#[derive(Default)]
struct Views {
    list: Vec<String>,
    seen: std::collections::HashSet<String>,
}

impl Views {
    fn push(&mut self, view: String) {
        if self.seen.insert(view.clone()) {
            self.list.push(view);
        }
    }
}

fn collect(command: &str, depth: usize, views: &mut Views) {
    for stmt in split_statements(command) {
        let Ok(argv) = shell_words::split(&stmt) else {
            views.push(stmt);
            continue;
        };
        let inner = strip_prefixes(&argv);
        if inner
            .first()
            .is_some_and(|w| PRINT_ONLY.contains(&base_name(w)))
        {
            // `echo rm -rf /` prints text; it is not a view.
            continue;
        }
        views.push(stmt);
        if depth < MAX_DEPTH {
            if let Some(split) = env_split_payload(&argv) {
                collect(&split, depth + 1, views);
            }
        }
        if inner.is_empty() {
            continue;
        }
        if inner.len() != argv.len() {
            // Re-quote so the stripped view keeps its argument boundaries.
            views.push(join_view(inner));
        }
        if depth < MAX_DEPTH {
            let payload = wrapper_payload(inner).or_else(|| {
                // Something was stripped: look past options it misread.
                (inner.len() != argv.len())
                    .then(|| payload_after_prefix(&argv))
                    .flatten()
            });
            if let Some(payload) = payload {
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
    fn stripped_view_keeps_home_bare() {
        let views = scan_views("sudo find ~ -type f -delete");
        assert!(
            views.contains(&"find ~ -type f -delete".to_string()),
            "{views:?}"
        );
        let views = scan_views("sudo chmod -R 777 $HOME");
        assert!(
            views.contains(&"chmod -R 777 $HOME".to_string()),
            "{views:?}"
        );
        let views = scan_views("sudo rm -rf 'my dir'");
        assert!(views.contains(&"rm -rf 'my dir'".to_string()), "{views:?}");
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
    fn no_statement_is_dropped_however_many_come_first() {
        let prefix: String = (0..500).map(|i| format!("true {i}; ")).collect();
        let cmd = format!("{prefix}bash -c 'rm -rf /'");
        assert!(scan_views(&cmd).contains(&"rm -rf /".to_string()));
    }

    #[test]
    fn finds_the_shell_after_prefix_options_with_operands() {
        for cmd in [
            "env -u FOO bash -c 'rm -rf /'",
            "timeout -s TERM 5 sh -c 'rm -rf /'",
            "timeout -k 5 10 bash -c 'rm -rf /'",
            "xargs -I {} bash -c 'rm -rf /'",
        ] {
            assert!(
                scan_views(cmd).contains(&"rm -rf /".to_string()),
                "{cmd:?} payload not found"
            );
        }
    }

    #[test]
    fn finds_the_payload_after_shell_options_and_env_split_string() {
        for cmd in [
            "bash -o pipefail -c 'rm -rf /'",
            "bash +o posix -c 'rm -rf /'",
            "bash -O extglob -c 'rm -rf /'",
            "bash -c -x 'rm -rf /'",
            "bash --norc -c 'rm -rf /'",
            "env -S 'bash -c \"rm -rf /\"'",
            "env --split-string='bash -c \"rm -rf /\"'",
            "env -u FOO -S 'bash -c \"rm -rf /\"'",
            "busybox sh -c 'rm -rf /'",
        ] {
            assert!(
                scan_views(cmd).contains(&"rm -rf /".to_string()),
                "{cmd:?} payload not found: {:?}",
                scan_views(cmd)
            );
        }
        // `env` is a wrapper only where it is the command.
        assert!(!scan_views("true env -S 'bash -c \"rm -rf /\"'").contains(&"rm -rf /".to_string()));
        assert!(scan_views("sudo env -S 'bash -c \"rm -rf /\"'").contains(&"rm -rf /".to_string()));
        // An option value is not the script.
        assert!(!scan_views("bash -c -o pipefail 'ls'").contains(&"pipefail".to_string()));
    }

    #[test]
    fn print_only_payload_statements_are_not_views() {
        for cmd in ["bash -c 'echo rm -rf /'", "sh -c \"printf '%s' rm -rf /\""] {
            let views = scan_views(cmd);
            assert!(
                !views.iter().any(|v| v.contains("rm -rf")),
                "{cmd:?} exposed a print argument: {views:?}"
            );
        }
        // A real command after a print in the same payload is still a view.
        assert!(scan_views("bash -c 'echo x; rm -rf /'").contains(&"rm -rf /".to_string()));
    }
}
