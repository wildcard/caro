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
/// Counts: `/abs`, `~/x`, `./x`, `../x`, `.dotfile`, `dir/`, `a/b.ext`, `name.ext`.
/// Does not count: `and/or`, `tcp/udp`, `24/7` (a slash alone is ambiguous).
fn is_path_token(t: &str, name: &Regex) -> bool {
    if t.is_empty() || t == "." || t == ".." || NOT_A_LITERAL.contains(&t.to_lowercase().as_str()) {
        return false;
    }
    let dotfile = t.starts_with('.') && t.chars().nth(1).is_some_and(|c| c.is_alphanumeric());
    t.starts_with(['/', '~'])
        || t.starts_with("./")
        || t.starts_with("../")
        || dotfile
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

/// True when `lit` occurs in `command` as a whole token: not inside a longer
/// word, number, path segment or file name (`80` is not in `8080`).
fn contains_token(command: &str, lit: &str) -> bool {
    let (command, lit) = (command.to_lowercase(), lit.to_lowercase());
    let word = |c: char| c.is_alphanumeric() || c == '_';
    command.match_indices(&lit).any(|(i, _)| {
        let before_ok = command[..i].chars().next_back().is_none_or(|c| !word(c));
        let mut after = command[i + lit.len()..].chars();
        let after_ok = match after.next() {
            None => true,
            Some('.') => after.next().is_none_or(|c| !word(c)),
            Some(c) => !word(c),
        };
        before_ok && after_ok
    })
}

/// Why `command` does not faithfully answer `query`, or `None` if it does.
pub(crate) fn slot_violation(query: &str, command: &str) -> Option<String> {
    for lit in query_literals(query) {
        if !contains_token(command, &lit) {
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
    fn literal_match_is_case_insensitive_and_whole_token() {
        assert!(slot_violation("count lines in README.MD", "wc -l README.md").is_none());
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
        assert!(slot_violation("show .env", "cat env").is_some());
        assert!(slot_violation("count lines in ./notes.txt", "wc -l ./notes.txt").is_none());
        assert!(slot_violation("count lines (old.txt)", "wc -l new.txt").is_some());
    }
}
