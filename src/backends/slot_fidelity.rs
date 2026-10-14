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

fn literal_regexes() -> &'static (Regex, Regex, Regex) {
    static RE: OnceLock<(Regex, Regex, Regex)> = OnceLock::new();
    RE.get_or_init(|| {
        (
            // Quoted text: 'my file.txt' or "my file.txt".
            Regex::new(r#"'([^']+)'|"([^"]+)""#).unwrap(),
            // Path or file-name token: has a slash, a leading ~ or ., or a
            // letter-led extension (notes.txt, café.md, v2/report.pdf).
            Regex::new(r"[^\s'\x22]*[/~][^\s'\x22]*|[^\s'\x22]*\p{L}\.[\p{L}0-9]{1,5}\b").unwrap(),
            // Port numbers: "port 3000", "port :8080".
            Regex::new(r"(?i)\bport\s*:?\s*(\d{1,5})\b").unwrap(),
        )
    })
}

fn trim_punct(s: &str) -> &str {
    s.trim_matches(|c: char| matches!(c, '.' | ',' | ';' | ':' | '?' | '!' | ')' | '('))
}

/// Explicit literals the user wrote that a faithful command must contain.
pub(crate) fn query_literals(query: &str) -> Vec<String> {
    let (quoted, path, port) = literal_regexes();
    let mut out: Vec<String> = Vec::new();
    let mut rest = query.to_string();

    for cap in quoted.captures_iter(query) {
        let lit = cap
            .get(1)
            .or_else(|| cap.get(2))
            .map(|m| m.as_str().to_string());
        if let Some(l) = lit {
            out.push(l);
        }
    }
    // Remove quoted text so its inner tokens are not counted twice.
    rest = quoted.replace_all(&rest, " ").into_owned();

    for m in path.find_iter(&rest) {
        let lit = trim_punct(m.as_str());
        if !lit.is_empty() {
            out.push(lit.to_string());
        }
    }
    for cap in port.captures_iter(query) {
        out.push(cap[1].to_string());
    }
    out.sort();
    out.dedup();
    out
}

/// Why `command` does not faithfully answer `query`, or `None` if it does.
pub(crate) fn slot_violation(query: &str, command: &str) -> Option<String> {
    for lit in query_literals(query) {
        if !command.contains(&lit) {
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
}
