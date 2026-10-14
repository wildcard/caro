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

/// File-type words and the proof that the filter reached the command. A proof
/// that starts with `.` must end an argument (`*.pdf`, `report.pdf`); any other
/// proof must start an argument (`python3`, `cargo`).
const TYPE_WORDS: &[(&str, &[&str])] = &[
    ("pdf", &[".pdf"]),
    ("python", &[".py", "python"]),
    ("rust", &[".rs", "cargo", "rustc"]),
    ("markdown", &[".md", ".markdown"]),
    ("yaml", &[".yaml", ".yml"]),
    ("json", &[".json", "jq"]),
    ("csv", &[".csv"]),
];

/// Capitalised words after "in" that are not a directory name.
const NOT_A_DIRECTORY: &[&str] = &[
    "python",
    "rust",
    "json",
    "yaml",
    "markdown",
    "csv",
    "pdf",
    "bash",
    "zsh",
    "fish",
    "linux",
    "macos",
    "git",
    "docker",
    "utc",
    "english",
    "gb",
    "mb",
    "kb",
    "tb",
    "ubuntu",
    "debian",
    "fedora",
    "centos",
    "unix",
    "windows",
    "production",
    "staging",
    "development",
    "prod",
    "dev",
    "ci",
    "kubernetes",
    "aws",
    "gcp",
    "azure",
    "iso",
    "rfc",
    "listen",
    "established",
    "apache",
    "nginx",
];

/// Lowercase directory names people say without a capital letter or "folder".
const KNOWN_DIRS: &[&str] = &[
    "downloads",
    "documents",
    "desktop",
    "pictures",
    "music",
    "videos",
];

/// Words after "in the ... folder" that name a place the template already means.
const IMPLIED_PLACE: &[&str] = &[
    "current", "this", "same", "working", "parent", "home", "that", "a", "an", "each", "every",
    "any", "all", "one", "your", "its", "their",
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
    lower_dest: Regex,
    size: Regex,
    types: Vec<Regex>,
    digits: Regex,
    port_in_cmd: Regex,
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
        // "port 3000", "ports 3000 or 3001", "tcp 22"; every number is a literal.
        port: Regex::new(r"(?i)\b(?:ports?\s*:?\s*|(?:tcp|udp)\s+)(\d{1,5}(?:\s*(?:,|or|and)\s*\d{1,5})*)\b").unwrap(),
        size: Regex::new(r"(?i)\b(\d+(?:\.\d+)?)\s*([kmgt])b?\b").unwrap(),
        lower_dest: Regex::new(r"(?i)\b(?:in|under|inside|within)\s+(?:the\s+|my\s+)?([a-z]+)\b").unwrap(),
        digits: Regex::new(r"\d+").unwrap(),
        port_in_cmd: Regex::new(r":(\d{2,5})\b").unwrap(),
        types: TYPE_WORDS
            .iter()
            .map(|(w, _)| Regex::new(&format!(r"\b{w}\s+files?\b")).unwrap())
            .collect(),
        colon_port: Regex::new(r"(?:^|[\s@])(?:localhost|[\w.-]+)?:(\d{2,5})\b").unwrap(),
        ip: Regex::new(r"\b(\d{1,3}(?:\.\d{1,3}){3})\b").unwrap(),
        pid: Regex::new(r"(?i)\b(?:pid|process(?:\s+id)?|kill)\s+(\d{1,7})\b").unwrap(),
        // "in Downloads", "under the Documents".
        cap_dest: Regex::new(r"\b(?:[Ii]n|[Uu]nder|[Ii]nside|[Ww]ithin)\s+(?:the\s+|my\s+)?(\p{Lu}[\p{L}\p{N}_.-]*)")
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
    /// A TCP/UDP port, which must not be satisfied by a `kill 3000` argument.
    pub port: bool,
}

/// Abbreviations that look like file names but are not.
const NOT_A_LITERAL: &[&str] = &["e.g", "i.e", "etc", "vs", "tar.gz", "tar.bz2", "tar.xz"];

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
        port: false,
    };
    let num = |t: &str| Literal {
        text: t.to_string(),
        numeric: true,
        port: false,
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
    for cap in re.port.captures_iter(query) {
        for n in re.digits.find_iter(&cap[1]) {
            out.push(Literal {
                port: true,
                ..num(n.as_str())
            });
        }
    }
    for cap in re.colon_port.captures_iter(query) {
        out.push(Literal {
            port: true,
            ..num(&cap[1])
        });
    }
    for r in [&re.pid, &re.ip] {
        for cap in r.captures_iter(query) {
            out.push(num(&cap[1]));
        }
    }
    // Sizes: "500MB" must reach the command as 500M, not as another number.
    for cap in re.size.captures_iter(query) {
        out.push(num(&format!("{}{}", &cap[1], cap[2].to_uppercase())));
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

/// The program a command runs and the operands it acts on. A leading `sudo`,
/// `env` or `time` is skipped. Printing programs (`echo`, `printf`) and no-ops
/// act on nothing, so they have no operands: `echo Documents` does not search
/// Documents. Scope: first program only; later stages of a pipeline are not
/// inspected, so a proof must appear in the first program's name or operands.
fn program_and_operands(tokens: &[String]) -> (&str, &[String]) {
    let skip = tokens
        .iter()
        .take_while(|t| matches!(t.as_str(), "sudo" | "env" | "time"))
        .count();
    let Some(program) = tokens.get(skip) else {
        return ("", &[]);
    };
    let operands = &tokens[skip + 1..];
    if matches!(program.as_str(), "echo" | "printf" | "true" | "false" | ":") {
        return (program, &[]);
    }
    (program, operands)
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
/// exact argument that is not an excluded or echoed mention. Numbers need a
/// non-word boundary and no trailing dot (`8.8.8.8.` is not `8.8.8.8`).
///
/// Scope: this does not parse a whole shell line. A literal that is the target
/// of a different sub-command (`wc -l y.txt; echo notes.txt`) is a known gap.
fn has_literal(command: &str, lit: &Literal) -> bool {
    if lit.numeric {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        if lit.port && shell_tokens(command).is_some_and(|t| t.first().is_some_and(|p| p == "kill"))
        {
            return false;
        }
        return command.match_indices(&lit.text).any(|(i, _)| {
            let before_ok = command[..i].chars().next_back().is_none_or(|c| !word(c));
            let after_ok = command[i + lit.text.len()..]
                .chars()
                .next()
                .is_none_or(|c| !word(c) && c != '.');
            before_ok && after_ok
        });
    }
    let want = normalize_path(&lit.text);
    let Some(tokens) = shell_tokens(command) else {
        return false;
    };
    // `echo notes.txt` prints the name; it has no operands, so it never matches.
    let (_, operands) = program_and_operands(&tokens);
    operands.iter().enumerate().any(|(i, t)| {
        let excluded = i > 0
            && (operands[i - 1].starts_with("--exclude")
                || operands[i - 1].starts_with("--ignore"));
        !excluded && normalize_path(t) == want
    })
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
    for cap in re.lower_dest.captures_iter(query) {
        let w = cap[1].to_string();
        if KNOWN_DIRS.contains(&w.as_str()) {
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
    let tokens = shell_tokens(command);
    // A destination is one whole argument or one whole path component, exact case.
    let in_place = |dest: &str| {
        tokens.as_ref().is_some_and(|ts| {
            program_and_operands(ts)
                .1
                .iter()
                .any(|t| t.split('/').any(|part| part == dest) || t == dest)
        })
    };
    for dest in destinations(query) {
        if !in_place(&dest) {
            return Some(format!(
                "destination `{dest}` is missing from the template command"
            ));
        }
    }
    let re = rx();
    for ((word, proofs), asked) in TYPE_WORDS.iter().zip(&re.types) {
        if asked.is_match(&q) {
            // A `.ext` proof must end an operand (a filter such as `*.pdf`); a
            // program proof (`python3`, `jq`) must start the program name.
            let proven = tokens.as_ref().is_some_and(|ts| {
                let (program, operands) = program_and_operands(ts);
                let program = program.rsplit('/').next().unwrap_or(program).to_lowercase();
                proofs.iter().any(|p| {
                    if p.starts_with('.') {
                        operands.iter().any(|t| t.to_lowercase().ends_with(p))
                    } else {
                        program.starts_with(p)
                    }
                })
            });
            if !proven {
                return Some(format!(
                    "file type `{word}` is missing from the template command"
                ));
            }
        }
    }
    // The command must not add a port the user did not name.
    let asked_ports: Vec<String> = query_literals(query)
        .into_iter()
        .filter(|l| l.port)
        .map(|l| l.text)
        .collect();
    for cap in re.port_in_cmd.captures_iter(command) {
        if !asked_ports.contains(&cap[1].to_string()) {
            return Some(format!(
                "template command adds the port `{}` the user did not name",
                &cap[1]
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

    #[test]
    fn destination_is_a_whole_argument_with_exact_case() {
        let q = "find files in Documents";
        assert!(slot_violation(q, "find Documents").is_none());
        assert!(slot_violation(q, "find ~/Documents/old").is_none());
        assert!(slot_violation(q, "find documents").is_some());
        assert!(slot_violation(q, "ls /home/DocumentsX").is_some());
        assert!(slot_violation(q, "find . -name Documents_backup").is_some());
        assert!(slot_violation("list files in the src folder", "ls SRC").is_some());
        assert!(slot_violation("list files in the src folder", "ls src").is_none());
    }

    #[test]
    fn lowercase_and_sentence_initial_destinations_are_found() {
        assert!(slot_violation(
            "find files larger than 10MB in downloads",
            "find . -size +10M"
        )
        .is_some());
        assert!(slot_violation(
            "In Downloads find files larger than 10MB",
            "find . -size +10M"
        )
        .is_some());
        assert!(slot_violation(
            "find files larger than 10MB in downloads",
            "find downloads -size +10M"
        )
        .is_none());
    }

    #[test]
    fn units_and_environment_words_are_not_directories() {
        assert!(slot_violation("show disk usage in GB", "df -h").is_none());
        assert!(slot_violation("show disk usage in MB sorted", "df -h").is_none());
        assert!(slot_violation("list processes in Production", "ps aux").is_none());
        assert!(slot_violation("show network interfaces in Ubuntu", "ip addr show").is_none());
    }

    #[test]
    fn type_word_needs_an_extension_or_program_argument() {
        let q = "find all pdf files";
        assert!(slot_violation(q, "find . -name '*.pdf'").is_none());
        assert!(slot_violation(q, "find . -name pdf_backup.txt").is_some());
        assert!(slot_violation(q, "find . -type f").is_some());
        assert!(slot_violation(q, "echo pdf").is_some());
        // The bare word without "file(s)" is not a file-type request.
        assert!(slot_violation("convert csv to json", "echo ok").is_none());
        assert!(slot_violation(
            "set up a Python virtual environment",
            "python3 -m venv .venv"
        )
        .is_none());
    }

    #[test]
    fn every_named_port_is_required_and_none_is_added() {
        assert!(slot_violation("use port 3000 or 3001", "lsof -i :3000").is_some());
        assert!(slot_violation("use port 3000 or 3001", "lsof -i :3000 -i :3001").is_none());
        assert!(slot_violation("what is on port 3000", "lsof -i :3000 -i :8080").is_some());
        assert!(slot_violation("show tcp 22", "ss -tn sport = :8080").is_some());
        assert!(slot_violation("kill the process on port 3000", "kill 3000").is_some());
    }

    #[test]
    fn ip_with_trailing_dot_is_a_different_value() {
        assert!(slot_violation("ping 8.8.8.8", "ping 8.8.8.8.").is_some());
        assert!(slot_violation("ping 8.8.8.8", "ping -c 4 8.8.8.8").is_none());
    }

    #[test]
    fn echoed_or_excluded_mention_is_not_the_target() {
        let q = "count lines in notes.txt";
        assert!(slot_violation(q, "echo notes.txt").is_some());
        assert!(slot_violation(q, "wc -l --exclude notes.txt y.txt").is_some());
        assert!(slot_violation(q, "wc -l notes.txt").is_none());
    }

    #[test]
    fn size_threshold_must_match() {
        assert!(
            slot_violation("find files larger than 500MB", "find . -type f -size +50M").is_some()
        );
        assert!(slot_violation(
            "find files larger than 3.5 GB",
            "find . -type f -size +100M"
        )
        .is_some());
        assert!(
            slot_violation("find files larger than 100MB", "find . -type f -size +100M").is_none()
        );
        assert!(slot_violation("find files larger than 1GB", "find . -type f -size +1G").is_none());
    }

    #[test]
    fn compound_extensions_formats_and_articles_are_not_literals_or_folders() {
        assert!(slot_violation(
            "create a tar.gz archive of the src directory",
            "tar -czf src.tar.gz src/"
        )
        .is_none());
        assert!(slot_violation("show current date in ISO format", "date +%Y-%m-%d").is_none());
        assert!(slot_violation("list ports in LISTEN state", "ss -tlnp").is_none());
        assert!(slot_violation("list all files in a directory", "ls -la").is_none());
        assert!(slot_violation("show disk usage of each folder", "du -sh */").is_none());
        // A named folder is still required.
        assert!(slot_violation("show disk usage of the logs folder", "du -sh */").is_some());
    }

    #[test]
    fn printing_programs_and_data_arguments_prove_nothing() {
        // Never executed: these are matcher inputs only.
        assert!(slot_violation("list files in Documents", "echo Documents").is_some());
        assert!(slot_violation("list files in Documents", "printf Documents").is_some());
        assert!(slot_violation("list files in Documents", "ls Documents").is_none());
        assert!(slot_violation("find all python files", "echo x.py").is_some());
        assert!(slot_violation("find all python files", "ls python_cache").is_some());
        assert!(slot_violation("find all python files", "find . -name '*.py' -type f").is_none());
        assert!(slot_violation("find all python files", "sudo find . -name '*.py'").is_none());
        assert!(slot_violation("run all python files tests", "python3 -m pytest").is_none());
        assert!(slot_violation("count lines in notes.txt", "true notes.txt").is_some());
    }
}
