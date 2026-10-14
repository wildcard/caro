//! Byte-preserving tokenizer and the closed request grammar.
//!
//! The parser is deterministic: every token has one role fixed by its
//! context, so there is no scan-order ambiguity. A request that the grammar
//! does not fully consume is declined (completeness rule). A listing needs a
//! file-selection subject (`files`, `file`, or an extension phrase);
//! filler-only or root-only input never compiles.

use std::fmt;

/// Byte span `[start, end)` of a token's inner text in the original request.
pub type Span = (usize, usize);

/// Hard cap on request length in bytes.
pub const MAX_REQUEST_BYTES: usize = 1000;
/// Window count bounds (inclusive).
pub const MAX_COUNT: u64 = 10_000;

/// The frozen public extension table: request word -> extension.
/// A word that is not listed here is a recognition miss. Do not extend it
/// without a design change.
pub const EXT_WORDS: &[(&str, &str)] = &[
    ("python", "py"),
    ("py", "py"),
    ("rust", "rs"),
    ("rs", "rs"),
    ("javascript", "js"),
    ("js", "js"),
    ("typescript", "ts"),
    ("ts", "ts"),
    ("markdown", "md"),
    ("md", "md"),
    ("json", "json"),
    ("yaml", "yaml"),
    ("yml", "yml"),
    ("toml", "toml"),
    ("xml", "xml"),
    ("html", "html"),
    ("css", "css"),
    ("csv", "csv"),
    ("log", "log"),
    ("pdf", "pdf"),
    ("txt", "txt"),
    ("text", "txt"),
    ("sql", "sql"),
    ("sh", "sh"),
];

/// Look up an extension word (case-insensitive ASCII).
pub fn ext_for_word(word: &str) -> Option<&'static str> {
    let w = word.to_ascii_lowercase();
    EXT_WORDS.iter().find(|(k, _)| *k == w).map(|(_, v)| *v)
}

/// A typed refusal. Never a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decline {
    pub code: &'static str,
    pub detail: String,
}

impl Decline {
    pub fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Decline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

/// Result of parsing one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Accept(RequestFacts),
    Clarify(String),
    Decline(Decline),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootFact {
    /// Exact source bytes (quotes removed for a quoted literal).
    pub source: String,
    /// Span in the request; `None` for the implicit current directory.
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// Exact basename, case-sensitive (`-name`).
    Name { name: String, span: Span },
    /// Extension reached through a table word; case-insensitive (`-iname`).
    ExtWord { ext: &'static str, span: Span },
    /// Explicit `*.ext` / `.ext` pattern; case-sensitive (`-name`).
    ExtPattern { ext: String, span: Span },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Minute,
    Hour,
    Day,
    Week,
}

impl Unit {
    pub fn seconds(self) -> i64 {
        match self {
            Unit::Minute => 60,
            Unit::Hour => 3600,
            Unit::Day => 86_400,
            Unit::Week => 604_800,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Unit::Minute => "minute",
            Unit::Hour => "hour",
            Unit::Day => "day",
            Unit::Week => "week",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowFact {
    pub count: u64,
    pub unit: Unit,
    /// Span of the count token; `None` for an implicit "one".
    pub count_span: Option<Span>,
    pub unit_span: Span,
}

/// What the request said, before any hygiene or arithmetic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestFacts {
    pub root: RootFact,
    pub filter: Option<Filter>,
    pub window: Option<WindowFact>,
}

#[derive(Debug, Clone)]
struct Tok {
    quoted: bool,
    text: String,
    span: Span,
}

impl Tok {
    fn lower(&self) -> Option<String> {
        (!self.quoted).then(|| self.text.to_ascii_lowercase())
    }
}

fn is_bidi(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | '\u{061C}')
}

fn is_quote(c: char) -> bool {
    matches!(c, '"' | '\'' | '`')
}

fn tokenize(input: &str) -> Result<Vec<Tok>, Decline> {
    if input.len() > MAX_REQUEST_BYTES {
        return Err(Decline::new("RequestTooLong", "request exceeds 1000 bytes"));
    }
    for c in input.chars() {
        if c == '\0' {
            return Err(Decline::new("NulByte", "request contains a NUL byte"));
        }
        if is_bidi(c) {
            return Err(Decline::new(
                "BidiControl",
                "request contains a bidi control",
            ));
        }
        if c.is_control() && !matches!(c, ' ' | '\t' | '\n') {
            return Err(Decline::new(
                "ControlCharacter",
                "request contains a control character",
            ));
        }
    }
    let mut toks = Vec::new();
    let mut it = input.char_indices().peekable();
    while let Some(&(i, c)) = it.peek() {
        if matches!(c, ' ' | '\t' | '\n') {
            it.next();
            continue;
        }
        if is_quote(c) {
            let rest = &input[i + c.len_utf8()..];
            let Some(close) = rest.find(c) else {
                return Err(Decline::new("UnbalancedQuote", "a quote is not closed"));
            };
            let start = i + c.len_utf8();
            let end = start + close;
            let inner = &input[start..end];
            if inner.is_empty() {
                return Err(Decline::new("EmptyLiteral", "a quoted literal is empty"));
            }
            if inner.contains(['\t', '\n']) {
                return Err(Decline::new(
                    "LiteralUnsupported",
                    "a quoted literal contains a tab or newline",
                ));
            }
            let after = end + c.len_utf8();
            if let Some(n) = input[after..].chars().next() {
                if !matches!(n, ' ' | '\t' | '\n') {
                    return Err(Decline::new(
                        "QuoteAdjacent",
                        "a quoted literal must be a separate token",
                    ));
                }
            }
            toks.push(Tok {
                quoted: true,
                text: inner.to_string(),
                span: (start, end),
            });
            while it.peek().is_some_and(|&(j, _)| j < after) {
                it.next();
            }
        } else {
            let start = i;
            let mut end = i;
            while let Some(&(j, d)) = it.peek() {
                if matches!(d, ' ' | '\t' | '\n') {
                    break;
                }
                if is_quote(d) {
                    return Err(Decline::new(
                        "QuoteInWord",
                        "a quote inside a word is not supported",
                    ));
                }
                end = j + d.len_utf8();
                it.next();
            }
            toks.push(Tok {
                quoted: false,
                text: input[start..end].to_string(),
                span: (start, end),
            });
        }
    }
    Ok(toks)
}

const LEADS: &[&str] = &["find", "search", "locate", "list", "show", "display", "get"];
const DETS: &[&str] = &["all", "every", "the", "a", "an", "my", "any"];
const FILLERS: &[&str] = &[
    "that",
    "which",
    "were",
    "was",
    "are",
    "recursively",
    "please",
];
const PREPS: &[&str] = &["in", "under", "inside", "within", "from", "at"];
const VERBS: &[&str] = &["modified", "edited", "updated"];
const NAME_WORDS: &[&str] = &["named", "called", "matching"];
const DIR_NOUNS: &[&str] = &["folder", "directory", "dir"];
const SPAN_PREPS: &[&str] = &["in", "within", "during", "over"];

fn is_vocab(w: &str) -> bool {
    LEADS.contains(&w)
        || DETS.contains(&w)
        || FILLERS.contains(&w)
        || PREPS.contains(&w)
        || VERBS.contains(&w)
        || NAME_WORDS.contains(&w)
        || DIR_NOUNS.contains(&w)
        || matches!(
            w,
            "files"
                | "file"
                | "last"
                | "past"
                | "me"
                | "for"
                | "have"
                | "been"
                | "here"
                | "this"
                | "that"
                | "current"
        )
}

fn count_word(w: &str) -> Option<u64> {
    Some(match w {
        "one" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        _ => return None,
    })
}

fn unit_word(w: &str) -> Option<Unit> {
    Some(match w {
        "minute" | "minutes" => Unit::Minute,
        "hour" | "hours" => Unit::Hour,
        "day" | "days" => Unit::Day,
        "week" | "weeks" => Unit::Week,
        _ => return None,
    })
}

/// Parse a count token: digits `1..=10000` or the words one..ten.
pub fn parse_count(w: &str) -> Option<u64> {
    if !w.is_empty() && w.bytes().all(|b| b.is_ascii_digit()) {
        let n: u64 = w.parse().ok()?;
        return (1..=MAX_COUNT).contains(&n).then_some(n);
    }
    count_word(&w.to_ascii_lowercase())
}

const CLARIFY_TEXT: &str = "Which time window do you mean? Say, for example, 'modified in the last 24 hours' (a rolling window) or 'modified in the last 7 days'. Calendar windows such as today or this week are not supported yet.";

fn clarify() -> Box<Outcome> {
    Box::new(Outcome::Clarify(CLARIFY_TEXT.to_string()))
}

fn decline(code: &'static str, detail: impl Into<String>) -> Box<Outcome> {
    Box::new(Outcome::Decline(Decline::new(code, detail)))
}

/// Cue scan over unquoted words, in token order. First cue wins.
fn scan_cues(toks: &[Tok]) -> Option<Box<Outcome>> {
    for (i, t) in toks.iter().enumerate() {
        let Some(w) = t.lower() else { continue };
        let next = toks.get(i + 1).and_then(Tok::lower);
        let code = match w.as_str() {
            "today" | "yesterday" | "tonight" | "since" | "recently" | "lately" | "few"
            | "while" => return Some(clarify()),
            "this" if matches!(next.as_deref(), Some("week" | "month" | "year")) => {
                return Some(clarify())
            }
            "size" | "larger" | "bigger" | "smaller" | "bytes" | "kb" | "mb" | "gb" | "empty" => {
                "SizeUnsupported"
            }
            "only" | "depth" | "level" | "levels" | "shallow" | "immediate" => "DepthUnsupported",
            "sort" | "sorted" | "order" | "ordered" | "newest" | "oldest" | "latest"
            | "biggest" | "largest" | "smallest" => "OrderUnsupported",
            "count" | "many" | "number" | "total" => "CountUnsupported",
            "not" | "except" | "without" | "excluding" | "but" | "other" => "NegationUnsupported",
            "symlink" | "symlinks" | "link" | "links" | "follow" => "LinksUnsupported",
            "delete" | "remove" | "move" | "copy" | "rename" | "write" | "kill" | "chmod"
            | "chown" => "WriteVerbUnsupported",
            "changed" | "created" | "accessed" | "opened" | "added" | "new" | "touched"
            | "access" | "creation" => "TimestampKindUnsupported",
            "directories" | "folders" | "dirs" => "NotRegularFiles",
            "containing" | "contains" | "contain" | "content" | "grep" => "ContentUnsupported",
            "older" | "before" | "after" | "between" | "earlier" | "ago" => {
                "WindowShapeUnsupported"
            }
            "month" | "months" | "year" | "years" => "UnitUnsupported",
            _ => continue,
        };
        return Some(decline(
            code,
            format!("'{}' is outside the supported scope", t.text),
        ));
    }
    None
}

struct Parser {
    toks: Vec<Tok>,
    i: usize,
    root: Option<RootFact>,
    filter: Option<Filter>,
    window: Option<WindowFact>,
    subject: bool,
}

impl Parser {
    fn peek(&self, off: usize) -> Option<&Tok> {
        self.toks.get(self.i + off)
    }

    fn peek_lower(&self, off: usize) -> Option<String> {
        self.peek(off).and_then(Tok::lower)
    }

    fn set_filter(&mut self, f: Filter) -> Result<(), Box<Outcome>> {
        if self.filter.is_some() {
            return Err(decline(
                "MultipleFilters",
                "only one name or extension filter",
            ));
        }
        self.filter = Some(f);
        Ok(())
    }

    fn set_root(&mut self, r: RootFact) -> Result<(), Box<Outcome>> {
        if self.root.is_some() {
            return Err(decline("MultipleRoots", "only one root is supported"));
        }
        self.root = Some(r);
        Ok(())
    }

    fn lead(&mut self) {
        if let Some(w) = self.peek_lower(0) {
            if LEADS.contains(&w.as_str()) {
                self.i += 1;
                if w == "search" && self.peek_lower(0).as_deref() == Some("for") {
                    self.i += 1;
                }
                if self.peek_lower(0).as_deref() == Some("me") {
                    self.i += 1;
                }
                if self.peek_lower(0).as_deref() == Some("please") {
                    self.i += 1;
                }
            }
        }
    }

    fn run(&mut self) -> Result<(), Box<Outcome>> {
        self.lead();
        while self.i < self.toks.len() {
            let t = self.toks[self.i].clone();
            if t.quoted {
                return Err(decline(
                    "AmbiguousLiteral",
                    "a quoted literal needs 'named', 'called', 'matching' or 'in' before it",
                ));
            }
            let w = t.text.to_ascii_lowercase();
            let wl = w.as_str();
            if DETS.contains(&wl) || FILLERS.contains(&wl) {
                self.i += 1;
            } else if wl == "have" {
                if self.peek_lower(1).as_deref() == Some("been") {
                    self.i += 2;
                } else {
                    return Err(residual(&t));
                }
            } else if wl == "files" || wl == "file" {
                self.subject = true;
                self.i += 1;
            } else if let Some(ext) = ext_for_word(&w)
                .filter(|_| matches!(self.peek_lower(1).as_deref(), Some("files" | "file")))
            {
                self.set_filter(Filter::ExtWord { ext, span: t.span })?;
                self.subject = true;
                self.i += 2;
            } else if let Some(ext) = t.text.strip_prefix("*.") {
                let ext = ext.to_string();
                self.set_filter(Filter::ExtPattern { ext, span: t.span })?;
                self.subject = true;
                self.i += 1;
                if matches!(self.peek_lower(0).as_deref(), Some("files" | "file")) {
                    self.i += 1;
                }
            } else if t.text.starts_with('.')
                && t.text.len() > 1
                && matches!(self.peek_lower(1).as_deref(), Some("files" | "file"))
            {
                let ext = t.text[1..].to_string();
                self.set_filter(Filter::ExtPattern { ext, span: t.span })?;
                self.subject = true;
                self.i += 2;
            } else if NAME_WORDS.contains(&wl) {
                self.name_phrase()?;
            } else if PREPS.contains(&wl) {
                self.root_phrase()?;
            } else if VERBS.contains(&wl) {
                self.window_phrase()?;
            } else if wl == "here" {
                self.set_root(RootFact {
                    source: ".".into(),
                    span: None,
                })?;
                self.i += 1;
            } else if matches!(wl, "this" | "current")
                && self
                    .peek_lower(1)
                    .is_some_and(|n| DIR_NOUNS.contains(&n.as_str()))
            {
                self.set_root(RootFact {
                    source: ".".into(),
                    span: None,
                })?;
                self.i += 2;
            } else {
                return Err(residual(&t));
            }
        }
        Ok(())
    }

    fn name_phrase(&mut self) -> Result<(), Box<Outcome>> {
        let Some(n) = self.peek(1).cloned() else {
            return Err(decline("MissingName", "a name must follow 'named'"));
        };
        if let Some(l) = n.lower() {
            if is_vocab(&l) {
                return Err(decline("MissingName", "a name must follow 'named'"));
            }
        }
        self.set_filter(Filter::Name {
            name: n.text,
            span: n.span,
        })?;
        self.i += 2;
        Ok(())
    }

    fn root_phrase(&mut self) -> Result<(), Box<Outcome>> {
        let mut j = self.i + 1;
        while self
            .toks
            .get(j)
            .and_then(Tok::lower)
            .is_some_and(|w| DETS.contains(&w.as_str()))
        {
            j += 1;
        }
        let Some(t) = self.toks.get(j).cloned() else {
            return Err(decline(
                "MissingRoot",
                "a folder must follow the preposition",
            ));
        };
        let lw = t.lower();
        if matches!(lw.as_deref(), Some("last" | "past")) {
            return Err(decline(
                "WindowWithoutVerb",
                "a time window needs 'modified', 'edited' or 'updated' before it",
            ));
        }
        let (root, consumed_to) = if matches!(lw.as_deref(), Some("this" | "that" | "current"))
            && self
                .toks
                .get(j + 1)
                .and_then(Tok::lower)
                .is_some_and(|n| DIR_NOUNS.contains(&n.as_str()))
        {
            (
                RootFact {
                    source: ".".into(),
                    span: None,
                },
                j + 2,
            )
        } else if lw.as_deref() == Some("here") {
            (
                RootFact {
                    source: ".".into(),
                    span: None,
                },
                j + 1,
            )
        } else {
            if let Some(l) = &lw {
                if is_vocab(l) {
                    return Err(decline(
                        "MissingRoot",
                        format!(
                            "'{}' is not a folder name; quote it to use it as one",
                            t.text
                        ),
                    ));
                }
            }
            let mut k = j + 1;
            if self
                .toks
                .get(k)
                .and_then(Tok::lower)
                .is_some_and(|n| DIR_NOUNS.contains(&n.as_str()))
            {
                k += 1;
            }
            (
                RootFact {
                    source: t.text.clone(),
                    span: Some(t.span),
                },
                k,
            )
        };
        self.set_root(root)?;
        self.i = consumed_to;
        Ok(())
    }

    fn window_phrase(&mut self) -> Result<(), Box<Outcome>> {
        if self.window.is_some() {
            return Err(decline(
                "MultipleWindows",
                "only one time window is supported",
            ));
        }
        let bad = || {
            decline(
                "WindowShapeUnsupported",
                "use 'modified in the last N days'",
            )
        };
        let mut j = self.i + 1;
        let lw = |j: usize, p: &Parser| p.toks.get(j).and_then(Tok::lower);
        if lw(j, self).is_some_and(|w| SPAN_PREPS.contains(&w.as_str())) {
            j += 1;
        }
        if lw(j, self).as_deref() == Some("the") {
            j += 1;
        }
        if !matches!(lw(j, self).as_deref(), Some("last" | "past")) {
            return Err(bad());
        }
        j += 1;
        let (count, count_span, unit_tok) = match lw(j, self) {
            Some(w) if unit_word(&w).is_some() => (None, None, j),
            Some(_) => {
                let t = &self.toks[j];
                let Some(n) = parse_count(&t.text) else {
                    return Err(bad());
                };
                (Some(n), Some(t.span), j + 1)
            }
            None => return Err(bad()),
        };
        let Some(ut) = self.toks.get(unit_tok) else {
            return Err(bad());
        };
        let Some(unit) = ut.lower().as_deref().and_then(unit_word) else {
            return Err(bad());
        };
        let count = match count {
            Some(n) => n,
            // Implicit "one" is accepted for minute and hour only.
            None => match unit {
                Unit::Minute | Unit::Hour => 1,
                Unit::Day | Unit::Week => return Err(clarify()),
            },
        };
        self.window = Some(WindowFact {
            count,
            unit,
            count_span,
            unit_span: ut.span,
        });
        self.i = unit_tok + 1;
        Ok(())
    }
}

fn residual(t: &Tok) -> Box<Outcome> {
    decline(
        "Residual",
        format!("'{}' is not understood in this request", t.text),
    )
}

/// Parse a request into facts, a clarification, or a typed decline.
pub fn parse(request: &str) -> Outcome {
    let toks = match tokenize(request) {
        Ok(t) => t,
        Err(d) => return Outcome::Decline(d),
    };
    if toks.is_empty() {
        return *decline("Empty", "the request is empty");
    }
    if let Some(o) = scan_cues(&toks) {
        return *o;
    }
    let mut p = Parser {
        toks,
        i: 0,
        root: None,
        filter: None,
        window: None,
        subject: false,
    };
    if let Err(o) = p.run() {
        return *o;
    }
    if !p.subject {
        return *decline(
            "NoSubject",
            "say what to list, for example 'files' or 'python files'",
        );
    }
    Outcome::Accept(RequestFacts {
        root: p.root.unwrap_or(RootFact {
            source: ".".into(),
            span: None,
        }),
        filter: p.filter,
        window: p.window,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(s: &str) -> RequestFacts {
        match parse(s) {
            Outcome::Accept(f) => f,
            o => panic!("expected accept for {s:?}, got {o:?}"),
        }
    }

    fn code(s: &str) -> &'static str {
        match parse(s) {
            Outcome::Decline(d) => d.code,
            o => panic!("expected decline for {s:?}, got {o:?}"),
        }
    }

    #[test]
    fn public_regressions_parse() {
        let f = facts("find python files modified in the last 7 days");
        assert_eq!(f.root.source, ".");
        assert!(matches!(f.filter, Some(Filter::ExtWord { ext: "py", .. })));
        let w = f.window.unwrap();
        assert_eq!((w.count, w.unit), (7, Unit::Day));

        let f = facts("list all python files");
        assert!(matches!(f.filter, Some(Filter::ExtWord { ext: "py", .. })));
        assert!(f.window.is_none());

        let f = facts("list all PDF files");
        assert!(matches!(f.filter, Some(Filter::ExtWord { ext: "pdf", .. })));
    }

    #[test]
    fn subject_is_required() {
        assert_eq!(code("please"), "NoSubject");
        assert_eq!(code("in docs"), "NoSubject");
        assert_eq!(code("find me"), "NoSubject");
        assert_eq!(code("named foo"), "NoSubject");
        assert_eq!(code("modified in the last 7 days"), "NoSubject");
    }

    #[test]
    fn root_name_and_pattern_forms() {
        let f = facts("find files in the docs folder");
        assert_eq!(f.root.source, "docs");
        let f = facts("find files under \"my notes\"");
        assert_eq!(f.root.source, "my notes");
        let f = facts("list files in ./src/");
        assert_eq!(f.root.source, "./src/");
        let f = facts("files named .env");
        assert!(matches!(f.filter, Some(Filter::Name { ref name, .. }) if name == ".env"));
        let f = facts("find *.py files");
        assert!(matches!(f.filter, Some(Filter::ExtPattern { ref ext, .. }) if ext == "py"));
        let f = facts("find .Py files");
        assert!(matches!(f.filter, Some(Filter::ExtPattern { ref ext, .. }) if ext == "Py"));
        let f = facts("find files in this folder");
        assert_eq!(f.root.source, ".");
    }

    #[test]
    fn quoted_bytes_are_preserved() {
        let s = "find files named \"a  b\" in 'x  y'";
        let f = facts(s);
        assert!(matches!(f.filter, Some(Filter::Name { ref name, .. }) if name == "a  b"));
        assert_eq!(f.root.source, "x  y");
        if let Some(Filter::Name { span, .. }) = f.filter {
            assert_eq!(&s[span.0..span.1], "a  b");
        }
        // cue words inside quotes are data
        let f = facts("find files named \"delete me.txt\" in \"old rm backups\"");
        assert_eq!(f.root.source, "old rm backups");
    }

    #[test]
    fn quote_and_control_hygiene() {
        assert_eq!(code("find files named \"abc"), "UnbalancedQuote");
        assert_eq!(code("find files named \"a\tb\""), "LiteralUnsupported");
        assert_eq!(code("find files named \"\""), "EmptyLiteral");
        assert_eq!(code("find files named don't"), "QuoteInWord");
        assert_eq!(code("find files named \"a\"b"), "QuoteAdjacent");
        assert_eq!(code("find files \u{0}"), "NulByte");
        assert_eq!(code("find files\r"), "ControlCharacter");
        assert_eq!(code("find files \u{202E}x"), "BidiControl");
        assert_eq!(code(&"a ".repeat(600)), "RequestTooLong");
        assert_eq!(code("find \"docs\" files"), "AmbiguousLiteral");
    }

    #[test]
    fn multiples_and_residual_decline() {
        assert_eq!(code("find python files in docs in src"), "MultipleRoots");
        assert_eq!(code("find python files named x"), "MultipleFilters");
        assert_eq!(
            code("find files modified in the last 2 days edited in the last 3 days"),
            "MultipleWindows"
        );
        assert_eq!(code("find files frobnicate"), "Residual");
        let f = facts("find rust files");
        assert!(matches!(f.filter, Some(Filter::ExtWord { ext: "rs", .. })));
    }

    #[test]
    fn window_forms() {
        let w = |s: &str| facts(s).window.unwrap();
        assert_eq!(
            (w("files modified in the last 3 hours").count, Unit::Hour),
            (3, Unit::Hour)
        );
        assert_eq!(w("files updated within the past two weeks").count, 2);
        assert_eq!(w("files edited last 30 minutes").unit, Unit::Minute);
        // implicit one: minute and hour only
        let h = w("files modified in the last hour");
        assert_eq!((h.count, h.unit, h.count_span), (1, Unit::Hour, None));
        let m = w("files modified in the past minute");
        assert_eq!((m.count, m.unit), (1, Unit::Minute));
        assert!(matches!(
            parse("files modified in the last day"),
            Outcome::Clarify(_)
        ));
        assert!(matches!(
            parse("files modified in the last week"),
            Outcome::Clarify(_)
        ));
        // bounds
        assert_eq!(w("files modified in the last 10000 minutes").count, 10_000);
        assert_eq!(
            code("files modified in the last 10001 minutes"),
            "WindowShapeUnsupported"
        );
        assert_eq!(
            code("files modified in the last 0 days"),
            "WindowShapeUnsupported"
        );
        assert_eq!(
            code("files modified in the last 7 months"),
            "UnitUnsupported"
        );
        assert_eq!(
            code("files modified in the last 7 years"),
            "UnitUnsupported"
        );
        assert_eq!(code("files in the last 7 days"), "WindowWithoutVerb");
        assert_eq!(
            code("files in docs from the last 7 days"),
            "WindowWithoutVerb"
        );
    }

    #[test]
    fn calendar_and_vague_clarify() {
        for s in [
            "files modified today",
            "files modified yesterday",
            "files modified tonight",
            "files modified this week",
            "files modified this month",
            "files modified since monday",
            "files modified recently",
            "files modified lately",
            "files modified a few days ago",
            "files modified a while back",
        ] {
            assert!(matches!(parse(s), Outcome::Clarify(_)), "{s}");
        }
    }

    #[test]
    fn unsupported_cues_decline() {
        let cases = [
            ("python files larger than 1MB", "SizeUnsupported"),
            ("python files only in this folder", "DepthUnsupported"),
            ("python files sorted by name", "OrderUnsupported"),
            ("count python files", "CountUnsupported"),
            ("python files not named test", "NegationUnsupported"),
            ("python files following symlinks", "LinksUnsupported"),
            ("delete python files", "WriteVerbUnsupported"),
            (
                "python files created in the last 3 days",
                "TimestampKindUnsupported",
            ),
            ("directories named src", "NotRegularFiles"),
            ("python files containing foo", "ContentUnsupported"),
            ("python files older than 7 days", "WindowShapeUnsupported"),
        ];
        for (s, c) in cases {
            assert_eq!(code(s), c, "{s}");
        }
    }

    #[test]
    fn ext_table_is_frozen() {
        assert_eq!(EXT_WORDS.len(), 24);
        assert_eq!(ext_for_word("PYTHON"), Some("py"));
        assert_eq!(ext_for_word("text"), Some("txt"));
        assert_eq!(ext_for_word("cobol"), None);
        assert_eq!(code("list cobol files"), "Residual");
    }
}
