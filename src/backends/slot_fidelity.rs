//! Slot fidelity gate for template (static-matcher) answers.
//!
//! A template command is only an answer if it carries the values the user
//! gave. `wc -l README.md` is not an answer to "count lines in 'my file.txt'",
//! and `kill PID` is not an answer to "kill the process on port 3000". When a
//! template cannot bind the request, the matcher must fall through, so a
//! backend that can bind it (or the clarify path) gets the request instead.
//!
//! The gate is deterministic and offline. It checks two things:
//! 1. every explicit literal in the query (quoted text, a path, a file name
//!    with an extension, a port number) appears in the command;
//! 2. the command holds no stand-in value (`file.txt`, `PID`, `/path/to`)
//!    that the query did not write itself.

use regex::Regex;
use std::sync::OnceLock;

/// Stand-in values that templates use for "a value the user should supply".
const PLACEHOLDERS: &[&str] = &[
    "file.txt",
    "file1.txt",
    "file2.txt",
    "readme.md",
    "largefile.txt",
    "source.txt",
    "dest.txt",
    "/path/to",
    "directory/",
    "container_name",
    "process_name",
    "<filename>",
    " filename",
    " pid",
    "example.com",
    "user@host",
    " dest/",
];

struct Literals {
    quoted: Regex,
    name: Regex,
    port: Regex,
    ip: Regex,
    pid: Regex,
}

fn literal_regexes() -> &'static Literals {
    static RE: OnceLock<Literals> = OnceLock::new();
    RE.get_or_init(|| Literals {
        // Quoted text. The opening quote follows whitespace, "(" or the start
        // of the query, so the apostrophes in "what's" and "users'" are not
        // quotes. Only the opening edge is checked so adjacent quoted values
        // ('a.txt' 'b.txt') are both found.
        quoted: Regex::new(r#"(?:^|[\s(])(?:'([^']+)'|"([^"]+)")"#).unwrap(),
        // A name token with a letter-led extension (notes.txt, café.md, src/main.rs).
        name: Regex::new(r"\p{L}\.[\p{L}0-9]{1,5}$").unwrap(),
        port: Regex::new(r"(?i)\bport\s*:?\s*(\d{1,5})\b").unwrap(),
        ip: Regex::new(r"\b(\d{1,3}(?:\.\d{1,3}){3})\b").unwrap(),
        pid: Regex::new(r"(?i)\b(?:pid|process(?:\s+id)?)\s+(\d{1,7})\b").unwrap(),
    })
}

/// Abbreviations that look like file names but are not.
const NOT_A_LITERAL: &[&str] = &["e.g", "i.e", "etc", "vs"];

/// A whitespace-separated token that names a path or file.
///
/// Counts: `/abs`, `~/x`, `./x`, `../x`, `dir/`, `a/b.ext`, `name.ext`.
/// Does not count: `and/or`, `tcp/udp`, `24/7` (a slash alone is ambiguous) or
/// `.py` (a bare extension, not a file; dotfiles such as `.env` are a known gap).
fn is_path_token(t: &str, name: &Regex) -> bool {
    if t.is_empty() || t == "." || t == ".." || NOT_A_LITERAL.contains(&t.to_lowercase().as_str()) {
        return false;
    }
    t.starts_with(['/', '~'])
        || t.starts_with("./")
        || t.starts_with("../")
        || t.ends_with('/')
        || name.is_match(t)
}

/// Explicit literals the user wrote that a faithful command must contain.
pub(crate) fn query_literals(query: &str) -> Vec<String> {
    let re = literal_regexes();
    let mut out: Vec<String> = Vec::new();

    for cap in re.quoted.captures_iter(query) {
        if let Some(m) = cap.get(1).or_else(|| cap.get(2)) {
            out.push(m.as_str().to_string());
        }
    }
    // Remove quoted text so its inner tokens are not counted twice.
    let rest = re.quoted.replace_all(query, " ").into_owned();

    for tok in rest.split_whitespace() {
        let t = tok
            .trim_start_matches(['(', '\'', '"'])
            .trim_end_matches(['.', ',', ';', ':', '?', '!', ')', '\'', '"']);
        if is_path_token(t, &re.name) {
            out.push(t.to_string());
        }
    }
    for rx in [&re.port, &re.pid] {
        for cap in rx.captures_iter(query) {
            out.push(cap[1].to_string());
        }
    }
    for cap in re.ip.captures_iter(query) {
        out.push(cap[1].to_string());
    }
    out.sort();
    out.dedup();
    out
}

/// Split a command into shell-like tokens. Quotes group text, and the
/// characters `| & ; < > ( )` and whitespace separate tokens outside quotes.
/// A token after `=` (`of=/dev/x`) is also offered on its own.
fn shell_tokens(command: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let push = |cur: &mut String, out: &mut Vec<String>| {
        if !cur.is_empty() {
            if let Some((_, rhs)) = cur.split_once('=') {
                if !rhs.is_empty() {
                    out.push(rhs.to_string());
                }
            }
            out.push(std::mem::take(cur));
        }
    };
    for c in command.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => cur.push(c),
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c.is_whitespace() || "|&;<>()".contains(c) => push(&mut cur, &mut out),
            None => cur.push(c),
        }
    }
    push(&mut cur, &mut out);
    out
}

/// Equivalence we allow between a typed path and a command argument: one
/// leading `./` and a trailing `/` on a directory. Case and Unicode are exact.
fn normalize_path(p: &str) -> &str {
    let p = p.strip_prefix("./").unwrap_or(p);
    if p.len() > 1 {
        p.trim_end_matches('/')
    } else {
        p
    }
}

/// True when the command carries the user's literal as one exact argument.
/// Numbers (ports, IPs, process ids) need a non-word boundary instead, since
/// they sit inside arguments like `:3000`.
fn has_literal(command: &str, lit: &str) -> bool {
    if lit.chars().all(|c| c.is_ascii_digit() || c == '.') {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        return command.match_indices(lit).any(|(i, _)| {
            let before_ok = command[..i].chars().next_back().is_none_or(|c| !word(c));
            let mut after = command[i + lit.len()..].chars();
            let after_ok = match after.next() {
                None => true,
                Some('.') => after.next().is_none_or(|c| !word(c)),
                Some(c) => !word(c),
            };
            before_ok && after_ok
        });
    }
    let want = normalize_path(lit);
    shell_tokens(command)
        .iter()
        .any(|t| normalize_path(t) == want)
}

/// Why `command` does not faithfully answer `query`, or `None` if it does.
pub(crate) fn slot_violation(query: &str, command: &str) -> Option<String> {
    for lit in query_literals(query) {
        if !has_literal(command, &lit) {
            return Some(format!(
                "user value `{lit}` is missing from the template command"
            ));
        }
    }
    let q = query.to_lowercase();
    let c = format!(" {}", command.to_lowercase());
    for ph in PLACEHOLDERS {
        if c.contains(ph) && !q.contains(ph.trim()) {
            return Some(format!(
                "template command holds the stand-in `{}` the user did not write",
                ph.trim()
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_quoted_name_is_a_violation() {
        assert!(slot_violation("count lines in 'my file.txt'", "wc -l README.md").is_some());
    }

    #[test]
    fn unicode_and_leading_dash_names_must_be_kept() {
        assert!(slot_violation("count words in café notes.txt", "wc -w file.txt").is_some());
        assert!(slot_violation("count words in notes.txt", "wc -w notes.txt").is_none());
        assert!(slot_violation("count words in -rf.txt", "wc -w ./-rf.txt").is_none());
    }

    #[test]
    fn destination_and_port_must_be_kept() {
        assert!(slot_violation("find large files in ~/Downloads", "find . -size +100M").is_some());
        assert!(slot_violation("what is on port 3000", "lsof -i :8080").is_some());
        assert!(slot_violation("what is on port 3000", "lsof -i :3000").is_none());
    }

    #[test]
    fn stand_in_value_the_user_did_not_write_is_a_violation() {
        assert!(slot_violation("kill the process on port", "kill PID").is_some());
        assert!(slot_violation(
            "compress the src folder",
            "tar -czf archive.tar.gz directory/"
        )
        .is_some());
        assert!(slot_violation("count lines in README.md", "wc -l README.md").is_none());
    }

    #[test]
    fn plain_queries_pass() {
        assert!(slot_violation("list all files", "ls -la").is_none());
        assert!(slot_violation("show disk usage", "df -h").is_none());
        assert!(slot_violation(
            "find files larger than 100MB.",
            "find . -type f -size +100M"
        )
        .is_none());
    }

    #[test]
    fn apostrophes_and_ambiguous_slashes_are_not_literals() {
        assert!(query_literals("what's using the most disk in the user's home").is_empty());
        assert!(query_literals("show tcp/udp and/or 24/7 sockets, e.g. listening").is_empty());
    }

    #[test]
    fn literal_match_is_exact_case_and_whole_argument() {
        // Two distinct files on a case-sensitive file system.
        assert!(slot_violation("count lines in README.MD", "wc -l README.md").is_some());
        assert!(slot_violation("count lines in README.MD", "wc -l README.MD").is_none());
        assert!(
            slot_violation("count lines in 'Café Notes.txt'", "wc -l 'café notes.txt'").is_some()
        );
        assert!(
            slot_violation("count lines in 'Café Notes.txt'", "wc -l 'Café Notes.txt'").is_none()
        );
        assert!(slot_violation(
            "count lines in 'Café Notes.txt'",
            "wc -l \"Café Notes.txt\""
        )
        .is_none());
        assert!(slot_violation("what is on port 80", "lsof -i :8080").is_some());
        assert!(slot_violation("show files in /tmp", "ls /tmpx").is_some());
        assert!(slot_violation("count words in notes.txt", "wc -w old_notes.txt.bak").is_some());
        assert!(slot_violation("count words in notes.txt", "wc -w notes.txt").is_none());
    }

    #[test]
    fn ip_and_pid_are_literals() {
        assert!(slot_violation(
            "failed auth from IP 192.168.1.100",
            "grep failed /var/log/auth.log"
        )
        .is_some());
        assert!(slot_violation("kill process 1234", "kill 4321").is_some());
        assert!(slot_violation("kill process 1234", "kill 1234").is_none());
    }

    #[test]
    fn adjacent_literals_are_all_required() {
        assert!(slot_violation("delete 'a.txt' 'b.txt'", "rm a.txt").is_some());
        assert!(slot_violation("copy a.txt b.txt", "cp a.txt").is_some());
        assert!(slot_violation("list /var/log, /tmp", "ls /var/log").is_some());
        assert!(slot_violation("list /var/log, /tmp", "ls /var/log /tmp").is_none());
    }

    #[test]
    fn relative_dirs_dotfiles_and_parens_are_literals() {
        assert!(slot_violation("find src/main.rs", "wc -l main.rs").is_some());
        assert!(slot_violation("list files in my_dir/", "ls").is_some());
        assert!(slot_violation("count lines in ./notes.txt", "wc -l ./notes.txt").is_none());
        assert!(slot_violation("count lines (old.txt)", "wc -l new.txt").is_some());
    }

    #[test]
    fn descendant_or_other_directory_is_not_the_requested_target() {
        assert!(slot_violation("list /tmp", "ls /tmp/subdir").is_some());
        assert!(slot_violation("list /tmp", "ls /tmp").is_none());
        assert!(slot_violation("list /tmp", "ls /tmp/").is_none());
        assert!(slot_violation("count words in notes.txt", "wc -w /other/notes.txt").is_some());
        assert!(slot_violation("count words in notes.txt", "wc -w notes.txt/extra").is_some());
        assert!(slot_violation("count words in notes.txt", "wc -w ./notes.txt").is_none());
        assert!(slot_violation("count words in ./notes.txt", "wc -w notes.txt").is_none());
        assert!(slot_violation("list /var/log", "ls /var/log2").is_some());
    }

    #[test]
    fn shell_tokens_split_on_operators_and_quotes() {
        assert!(slot_violation("sort notes.txt", "cat notes.txt|sort").is_none());
        assert!(slot_violation("copy 'my file.txt'", "cp 'my file.txt' out").is_none());
        assert!(slot_violation("copy my.txt to /dev/x", "dd if=my.txt of=/dev/x").is_none());
    }
}
