//! Slot fidelity gate for template (static-matcher) answers.
//!
//! A template command is only an answer if it carries the values the user
//! gave. `wc -l README.md` is not an answer to "count lines in 'my file.txt'",
//! and `kill PID` is not an answer to "kill the process on port 3000". When a
//! template cannot bind the request, the matcher must fall through, so a
//! backend that can bind it (or the clarify path) gets the request instead.
//!
//! The gate is deterministic and offline. It checks four things:
//! 1. every explicit literal in the query (quoted text, a path, a file name
//!    with an extension, a port, an IP, a process id) is one exact argument of
//!    the command, with exact case and Unicode;
//! 2. a bare destination ("in Downloads", "the src folder") reaches the command;
//! 3. a file-type word ("PDF", "Python files") reaches the command;
//! 4. the command holds no stand-in value (`file.txt`, `PID`, `/path/to`)
//!    that the query did not write itself.
//!
//! This is intermediate correctness: it rejects known-wrong templates. It does
//! not make a declined request succeed.

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
    "old_text",
    "new_text",
    "<filename>",
    " filename",
    " pid",
    "example.com",
    "user@host",
    " dest/",
];

/// File-type words and the command text that proves the filter was kept.
const TYPE_WORDS: &[(&str, &[&str])] = &[
    ("pdf", &["pdf"]),
    ("python", &[".py", "python"]),
    ("rust", &[".rs", "rust", "cargo"]),
    ("markdown", &[".md", "markdown"]),
    ("yaml", &[".yaml", ".yml", "yaml"]),
    ("json", &[".json", "json"]),
    ("csv", &[".csv", "csv"]),
];

/// Capitalised words after "in" that are not a directory name.
const NOT_A_DIRECTORY: &[&str] = &[
    "python", "rust", "json", "yaml", "markdown", "csv", "pdf", "bash", "zsh", "fish", "linux",
    "macos", "git", "docker", "utc", "english",
];

/// Words after "in the ... folder" that name a place the template already means.
const IMPLIED_PLACE: &[&str] = &[
    "current", "this", "same", "working", "parent", "home", "that",
];

struct Rx {
    quoted: Regex,
    name: Regex,
    port: Regex,
    colon_port: Regex,
    ip: Regex,
    pid: Regex,
    cap_dest: Regex,
    named_dest: Regex,
}

fn rx() -> &'static Rx {
    static RE: OnceLock<Rx> = OnceLock::new();
    RE.get_or_init(|| Rx {
        // Quoted text. The opening quote follows whitespace, "(" or the start
        // of the query, so the apostrophes in "what's" and "users'" are not
        // quotes. Only the opening edge is checked so adjacent quoted values
        // ('a.txt' 'b.txt') are both found.
        quoted: Regex::new(r#"(?:^|[\s(])(?:'([^']+)'|"([^"]+)")"#).unwrap(),
        // A name token: any name char (letter, mark, digit, _ or -) before the
        // dot, then an extension that holds a letter (so 3.14 is not a file).
        name: Regex::new(r"[\p{L}\p{M}\p{N}_-]\.(?:[\p{N}]*[\p{L}\p{M}][\p{L}\p{M}\p{N}]*)$").unwrap(),
        port: Regex::new(r"(?i)\bport\s*:?\s*(\d{1,5})\b").unwrap(),
        colon_port: Regex::new(r"(?:^|[\s@])(?:localhost|[\w.-]+)?:(\d{2,5})\b").unwrap(),
        ip: Regex::new(r"\b(\d{1,3}(?:\.\d{1,3}){3})\b").unwrap(),
        pid: Regex::new(r"(?i)\b(?:pid|process(?:\s+id)?|kill)\s+(\d{1,7})\b").unwrap(),
        // "in Downloads", "under the Documents".
        cap_dest: Regex::new(r"\b(?:in|under|inside|within)\s+(?:the\s+|my\s+)?(\p{Lu}[\p{L}\p{N}_.-]*)")
            .unwrap(),
        // "in the src folder", "in logs directory".
        named_dest: Regex::new(
            r"(?i)\b(?:in|under|inside|within|of)\s+(?:the\s+|my\s+)?([\p{L}\p{N}_.-]+)\s+(?:folder|directory|dir)\b",
        )
        .unwrap(),
    })
}

/// An explicit value; `numeric` ones (ports, IPs, ids) sit inside arguments
/// such as `:3000`, so they match on a non-word boundary instead.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub(crate) struct Literal {
    pub text: String,
    pub numeric: bool,
}

/// Abbreviations that look like file names but are not.
const NOT_A_LITERAL: &[&str] = &["e.g", "i.e", "etc", "vs"];

/// A whitespace-separated token that names a path or file.
///
/// Counts: `/abs`, `~/x`, `./x`, `../x`, `dir/`, `a/b.ext`, `name.ext`.
/// Does not count: `and/or`, `tcp/udp`, `24/7` (a slash alone is ambiguous),
/// `.py` or `.tar.gz` (extension filters), `3.14`. Dotfiles such as `.env` and
/// extensionless names such as `Makefile` are a known gap.
fn is_path_token(t: &str, name: &Regex) -> bool {
    if t.is_empty() || t == "." || t == ".." || NOT_A_LITERAL.contains(&t.to_lowercase().as_str()) {
        return false;
    }
    if t.starts_with('.') && !t.starts_with("./") && !t.starts_with("../") {
        return false;
    }
    t.starts_with(['/', '~'])
        || t.starts_with("./")
        || t.starts_with("../")
        || t.ends_with('/')
        || name.is_match(t)
}

/// Explicit values the user wrote that a faithful command must contain.
pub(crate) fn query_literals(query: &str) -> Vec<Literal> {
    let re = rx();
    let mut out: Vec<Literal> = Vec::new();
    let text = |t: &str| Literal {
        text: t.to_string(),
        numeric: false,
    };
    let num = |t: &str| Literal {
        text: t.to_string(),
        numeric: true,
    };

    for cap in re.quoted.captures_iter(query) {
        if let Some(m) = cap.get(1).or_else(|| cap.get(2)) {
            out.push(text(m.as_str()));
        }
    }
    // Remove quoted text so its inner tokens are not counted twice.
    let rest = re.quoted.replace_all(query, " ").into_owned();

    for tok in rest.split_whitespace() {
        let t = tok
            .trim_start_matches(['(', '\'', '"'])
            .trim_end_matches(['.', ',', ';', ':', '?', '!', ')', '\'', '"']);
        if is_path_token(t, &re.name) {
            out.push(text(t));
        }
    }
    for r in [&re.port, &re.pid, &re.colon_port, &re.ip] {
        for cap in r.captures_iter(query) {
            out.push(num(&cap[1]));
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Split a command into shell-like tokens, or `None` if it cannot be parsed
/// (an unbalanced quote). Quotes group text, a backslash escapes one character,
/// and whitespace and `| & ; < > ( ) `` ` `` ` separate tokens outside quotes.
/// A value after `NAME=` (not `--opt=`) is also offered on its own, for
/// `dd of=/dev/x`.
fn shell_tokens(command: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut had_quote = false;
    let mut quote: Option<char> = None;
    let mut chars = command.chars();
    let flush = |cur: &mut String, had_quote: &mut bool, out: &mut Vec<String>| {
        if !cur.is_empty() || *had_quote {
            if let Some((lhs, rhs)) = cur.split_once('=') {
                let plain = lhs
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                    && lhs.chars().all(|c| c.is_alphanumeric() || c == '_');
                if plain && !rhs.is_empty() && !rhs.contains('=') {
                    out.push(rhs.to_string());
                }
            }
            out.push(std::mem::take(cur));
        }
        *had_quote = false;
    };
    while let Some(c) = chars.next() {
        match quote {
            Some(q) if c == q => quote = None,
            Some('"') if c == '\\' => match chars.next() {
                Some(n) if n == '"' || n == '\\' || n == '$' || n == '`' => cur.push(n),
                Some(n) => {
                    cur.push('\\');
                    cur.push(n);
                }
                None => return None,
            },
            Some(_) => cur.push(c),
            None if c == '\'' || c == '"' => {
                quote = Some(c);
                had_quote = true;
            }
            None if c == '\\' => cur.push(chars.next()?),
            None if c.is_whitespace() || "|&;<>()`".contains(c) => {
                flush(&mut cur, &mut had_quote, &mut out)
            }
            None => cur.push(c),
        }
    }
    if quote.is_some() {
        return None;
    }
    flush(&mut cur, &mut had_quote, &mut out);
    Some(out)
}

/// Equivalence we allow between a typed path and a command argument: one
/// leading `./` and one trailing `/` on a directory. Case and Unicode are exact.
fn normalize_path(p: &str) -> &str {
    let p = p.strip_prefix("./").unwrap_or(p);
    if p.is_empty() {
        return ".";
    }
    if p.len() > 1 {
        p.strip_suffix('/').unwrap_or(p)
    } else {
        p
    }
}

/// True when the command carries the user's value. Names and paths must be one
/// exact argument. Numbers need a non-word boundary.
fn has_literal(command: &str, lit: &Literal) -> bool {
    if lit.numeric {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        return command.match_indices(&lit.text).any(|(i, _)| {
            let before_ok = command[..i].chars().next_back().is_none_or(|c| !word(c));
            let mut after = command[i + lit.text.len()..].chars();
            let after_ok = match after.next() {
                None => true,
                Some('.') => after.next().is_none_or(|c| !word(c)),
                Some(c) => !word(c),
            };
            before_ok && after_ok
        });
    }
    let want = normalize_path(&lit.text);
    shell_tokens(command).is_some_and(|tokens| tokens.iter().any(|t| normalize_path(t) == want))
}

/// Bare destination words the user wrote ("Downloads", "src" in "the src folder").
fn destinations(query: &str) -> Vec<String> {
    let re = rx();
    let mut out = Vec::new();
    for cap in re.cap_dest.captures_iter(query) {
        let w = cap[1].trim_end_matches(['.', ',']).to_string();
        if !NOT_A_DIRECTORY.contains(&w.to_lowercase().as_str()) {
            out.push(w);
        }
    }
    for cap in re.named_dest.captures_iter(query) {
        let w = cap[1].to_string();
        if !IMPLIED_PLACE.contains(&w.to_lowercase().as_str()) {
            out.push(w);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Why `command` does not faithfully answer `query`, or `None` if it does.
pub(crate) fn slot_violation(query: &str, command: &str) -> Option<String> {
    for lit in query_literals(query) {
        if !has_literal(command, &lit) {
            return Some(format!(
                "user value `{}` is missing from the template command",
                lit.text
            ));
        }
    }
    let q = query.to_lowercase();
    let c = command.to_lowercase();
    for dest in destinations(query) {
        if !c.contains(&dest.to_lowercase()) {
            return Some(format!(
                "destination `{dest}` is missing from the template command"
            ));
        }
    }
    for (word, proofs) in TYPE_WORDS {
        let asked = Regex::new(&format!(r"\b{word}\b")).unwrap().is_match(&q);
        if asked && !proofs.iter().any(|p| c.contains(p)) {
            return Some(format!(
                "file type `{word}` is missing from the template command"
            ));
        }
    }
    let spaced = format!(" {c}");
    for ph in PLACEHOLDERS {
        if spaced.contains(ph) && !q.contains(ph.trim()) {
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

    #[test]
    fn names_ending_in_digit_or_underscore_and_long_extensions_are_literals() {
        assert!(slot_violation("count lines in report2.txt", "wc -l report.txt").is_some());
        assert!(slot_violation("count lines in data_2024.csv", "wc -l data.csv").is_some());
        assert!(slot_violation("open dump.sqlite3", "cat other.db").is_some());
        assert!(slot_violation("open notes.markdown", "cat notes.md").is_some());
        assert!(slot_violation("count lines in report2.txt", "wc -l report2.txt").is_none());
    }

    #[test]
    fn decomposed_unicode_name_is_a_literal() {
        assert!(slot_violation("count lines in e\u{301}.txt", "wc -l x.txt").is_some());
        assert!(slot_violation("count lines in e\u{301}.txt", "wc -l e\u{301}.txt").is_none());
    }

    #[test]
    fn option_values_are_not_the_requested_target() {
        assert!(slot_violation("count lines in x.txt", "wc -l --exclude=x.txt y.txt").is_some());
        assert!(
            slot_violation("backup secret.txt", "tar czf b.tgz --exclude=secret.txt .").is_some()
        );
        assert!(slot_violation("copy my.txt to /dev/x", "dd if=my.txt of=/dev/x").is_none());
    }

    #[test]
    fn only_one_trailing_slash_is_equivalent() {
        assert!(slot_violation("list /tmp", "ls /tmp//").is_some());
        assert!(slot_violation("list /tmp", "ls /tmp/").is_none());
        assert!(slot_violation("list files in ./", "ls .").is_none());
    }

    #[test]
    fn shell_escapes_backticks_and_unbalanced_quotes() {
        assert!(slot_violation("count lines in 'a b.txt'", "wc -l a\\ b.txt").is_none());
        assert!(slot_violation("count lines in notes.txt", "wc -l `echo notes.txt`").is_none());
        // An unbalanced quote cannot be parsed, so nothing is proven.
        assert!(slot_violation("count lines in notes.txt", "wc -l 'notes.txt").is_some());
    }

    #[test]
    fn relative_file_path_is_not_the_default_system_path() {
        let q = "count HTTP status codes in logs/access.log";
        assert!(slot_violation(q, "awk '{print $9}' /var/log/nginx/access.log").is_some());
        assert!(slot_violation(q, "awk '{print $9}' logs/access.log").is_none());
        assert!(slot_violation(q, "awk '{print $9}' ./logs/access.log").is_none());
        assert!(query_literals("tcp/udp and/or 24/7").is_empty());
    }

    #[test]
    fn bare_port_and_pid_numbers_are_literals() {
        assert!(slot_violation("what listens on :3000", "lsof -i :8080").is_some());
        assert!(slot_violation("what listens on :3000", "lsof -i :3000").is_none());
        assert!(slot_violation("kill 1234", "kill 4321").is_some());
    }

    #[test]
    fn extension_filters_and_versions_are_not_file_literals() {
        assert!(query_literals("find all .tar.gz files").is_empty());
        assert!(query_literals("upgrade to 3.14 now").is_empty());
    }

    #[test]
    fn bare_destination_must_be_kept() {
        let q = "find all PDF files larger than 10MB in Downloads";
        assert!(slot_violation(q, "find . -type f -size +10M").is_some());
        assert!(slot_violation(q, "find Downloads -type f -iname '*.pdf' -size +10M").is_none());
        assert!(slot_violation("list files in the Downloads folder", "ls -la").is_some());
        assert!(slot_violation("list files in the current directory", "ls -la").is_none());
        assert!(slot_violation(
            "search for TODO in Python files",
            "grep -rn TODO --include='*.py' ."
        )
        .is_none());
    }

    #[test]
    fn file_type_word_must_reach_the_command() {
        assert!(slot_violation("list all Python files", "ls -la").is_some());
        assert!(slot_violation(
            "find all rust files modified today",
            "find . -type f -mtime 0"
        )
        .is_some());
        assert!(slot_violation("find all python files", "find . -name \"*.py\" -type f").is_none());
        assert!(slot_violation(
            "replace tabs with spaces in python files",
            "sed -i 's/old_text/new_text/g' *.txt"
        )
        .is_some());
    }
}
