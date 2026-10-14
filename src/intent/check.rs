//! Hygiene and arithmetic: turns [`RequestFacts`] into a [`VerifiedIntent`].
//!
//! `VerifiedIntent` has no public constructor; `verify` is the only way to
//! get one, and `compile` accepts nothing else.

use super::facts::{
    ext_for_word, parse_count, Decline, Filter, RequestFacts, Unit, WindowFact, MAX_COUNT,
};

pub const MAX_ROOT_BYTES: usize = 4096;
pub const MAX_NAME_BYTES: usize = 255;
pub const MAX_EXT_BYTES: usize = 16;

/// Window bounds, both from the same captured whole-second `T0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub count: u64,
    pub unit: Unit,
    /// Window length in elapsed seconds (not calendar days).
    pub width_secs: i64,
    /// Exclusive lower bound `T0 - W`.
    pub cutoff: i64,
    /// Inclusive upper bound `T0`.
    pub t0: i64,
}

/// How the filter is spelled in `find`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// `-name '<exact>'`
    Name(String),
    /// `-iname '*.<ext>'`
    ExtInsensitive(String),
    /// `-name '*.<ext>'`
    ExtSensitive(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedIntent {
    root_source: String,
    root_emit: String,
    pattern: Option<Pattern>,
    window: Option<Window>,
}

impl VerifiedIntent {
    pub fn root_source(&self) -> &str {
        &self.root_source
    }
    pub fn root_emit(&self) -> &str {
        &self.root_emit
    }
    pub fn pattern(&self) -> Option<&Pattern> {
        self.pattern.as_ref()
    }
    pub fn window(&self) -> Option<&Window> {
        self.window.as_ref()
    }
}

/// Validate a root and compute the emitted form. Pure; reads no file system.
///
/// Accepted: `.`, `./`, `./x/...`, `/x/...`, and plain relative paths.
/// Rejected: empty, over-long, control characters, `~`, a trailing `/` (except
/// exactly `./`), a leading `//`, any `..` component, empty components, a `.`
/// component after the first, and any root made only of dots and slashes
/// other than `.` and `./` (this covers `/`, `/.`, `./.`, `..`, `...`).
pub fn emit_root(source: &str) -> Result<String, Decline> {
    let bad = |d: &str| Err(Decline::new("RootRejected", d.to_string()));
    if source.is_empty() {
        return bad("the folder is empty");
    }
    if source.len() > MAX_ROOT_BYTES {
        return bad("the folder path is too long");
    }
    if source.chars().any(|c| c.is_control()) {
        return bad("the folder contains a control character");
    }
    if source == "." || source == "./" {
        return Ok(".".to_string());
    }
    if source.bytes().all(|b| b == b'.' || b == b'/') {
        return bad("the folder is the filesystem root or a dot-only path");
    }
    if source.contains('~') {
        return bad("'~' is not expanded; give the path");
    }
    if source.ends_with('/') {
        return bad("a trailing '/' is not accepted (except './')");
    }
    if source.starts_with("//") {
        return bad("a leading '//' is not accepted");
    }
    let rel = !source.starts_with('/');
    let body = if rel { source } else { &source[1..] };
    for (i, comp) in body.split('/').enumerate() {
        match comp {
            "" => return bad("empty path component"),
            ".." => return bad("'..' is not accepted"),
            "." if !(rel && i == 0) => return bad("'.' component is not accepted"),
            _ => {}
        }
    }
    Ok(if !rel || source.starts_with("./") {
        source.to_string()
    } else {
        // Every other relative root gets './' so find cannot read it as an
        // expression (`-x`, `!`, `(`, `)`, `,`). Quoting only protects the shell.
        format!("./{source}")
    })
}

fn name_ok(name: &str) -> Result<(), Decline> {
    let bad = |d: &str| Err(Decline::new("NameRejected", d.to_string()));
    if name.is_empty() || name.len() > MAX_NAME_BYTES {
        return bad("a name must be 1 to 255 bytes");
    }
    if name.contains('/') {
        return Err(Decline::new("NameHasSlash", "a name cannot contain '/'"));
    }
    if name.contains(['*', '?', '[', ']', '\\']) {
        return bad("globs are not supported in an exact name");
    }
    if name == "." || name == ".." {
        return bad("'.' and '..' are not file names");
    }
    if name.chars().any(|c| c.is_control()) {
        return bad("a name cannot contain a control character");
    }
    Ok(())
}

fn ext_ok(ext: &str) -> Result<(), Decline> {
    let ok = !ext.is_empty()
        && ext.len() <= MAX_EXT_BYTES
        && ext
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-'));
    if ok {
        Ok(())
    } else {
        Err(Decline::new(
            "ExtensionRejected",
            "an extension is 1 to 16 letters, digits, '_', '+' or '-'",
        ))
    }
}

fn window_for(w: &WindowFact, t0: i64) -> Result<Window, Decline> {
    if !(1..=MAX_COUNT).contains(&w.count) {
        return Err(Decline::new("WindowCount", "count must be 1 to 10000"));
    }
    let width = i64::try_from(w.count)
        .ok()
        .and_then(|c| c.checked_mul(w.unit.seconds()))
        .ok_or_else(|| Decline::new("WindowOverflow", "window is too large"))?;
    if t0 <= 0 {
        return Err(Decline::new("ClockRange", "clock value is out of range"));
    }
    if width > t0 {
        return Err(Decline::new("WindowOverflow", "window reaches before 1970"));
    }
    Ok(Window {
        count: w.count,
        unit: w.unit,
        width_secs: width,
        cutoff: t0 - width,
        t0,
    })
}

/// Re-derive every value from its span in the original request and require
/// it to equal the parsed fact (source association).
fn associate(f: &RequestFacts, request: &str) -> Result<(), Decline> {
    let miss = || {
        Decline::new(
            "SourceMismatch",
            "a parsed value does not match the request",
        )
    };
    let at = |s: (usize, usize)| request.get(s.0..s.1).ok_or_else(miss);
    if let Some(s) = f.root.span {
        if at(s)? != f.root.source {
            return Err(miss());
        }
    } else if f.root.source != "." {
        return Err(miss());
    }
    match &f.filter {
        Some(Filter::Name { name, span }) if at(*span)? != name => return Err(miss()),
        Some(Filter::ExtWord { ext, span }) if ext_for_word(at(*span)?) != Some(*ext) => {
            return Err(miss())
        }
        Some(Filter::ExtPattern { ext, span }) => {
            let t = at(*span)?;
            if t.strip_prefix("*.").or_else(|| t.strip_prefix('.')) != Some(ext.as_str()) {
                return Err(miss());
            }
        }
        _ => {}
    }
    if let Some(w) = &f.window {
        if let Some(cs) = w.count_span {
            if parse_count(at(cs)?) != Some(w.count) {
                return Err(miss());
            }
        }
        let u = at(w.unit_span)?.to_ascii_lowercase();
        if !u.starts_with(w.unit.name()) {
            return Err(miss());
        }
    }
    Ok(())
}

/// Check facts against the original request and the single captured `t0`.
pub fn verify(facts: &RequestFacts, request: &str, t0: i64) -> Result<VerifiedIntent, Decline> {
    associate(facts, request)?;
    let root_emit = emit_root(&facts.root.source)?;
    let pattern = match &facts.filter {
        None => None,
        Some(Filter::Name { name, .. }) => {
            name_ok(name)?;
            Some(Pattern::Name(name.clone()))
        }
        Some(Filter::ExtWord { ext, .. }) => {
            ext_ok(ext)?;
            Some(Pattern::ExtInsensitive((*ext).to_string()))
        }
        Some(Filter::ExtPattern { ext, .. }) => {
            ext_ok(ext)?;
            Some(Pattern::ExtSensitive(ext.clone()))
        }
    };
    let window = facts
        .window
        .as_ref()
        .map(|w| window_for(w, t0))
        .transpose()?;
    Ok(VerifiedIntent {
        root_source: facts.root.source.clone(),
        root_emit,
        pattern,
        window,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::facts::{parse, Outcome};

    const T0: i64 = 1_760_000_000;

    fn v(req: &str) -> Result<VerifiedIntent, Decline> {
        match parse(req) {
            Outcome::Accept(f) => verify(&f, req, T0),
            o => panic!("not accepted: {o:?}"),
        }
    }

    #[test]
    fn root_emit_table() {
        for (src, want) in [
            (".", "."),
            ("./", "."),
            ("docs", "./docs"),
            ("src/lib", "./src/lib"),
            ("./src", "./src"),
            ("/abs/dir", "/abs/dir"),
            ("-x", "./-x"),
            ("!", "./!"),
            ("(", "./("),
            (")", "./)"),
            (",", "./,"),
            ("my notes", "./my notes"),
        ] {
            assert_eq!(emit_root(src).unwrap(), want, "{src}");
        }
    }

    #[test]
    fn root_rejections() {
        for src in [
            "", "/", "//", "//x", "/.", "./.", "..", "../", "...", "....", "../x", "a/../b", "x/",
            "docs/", "./src/", "/abs/", "~", "~/x", "a//b", "a/./b", "/./a", "a\tb", "a\nb",
            "a\u{7f}b",
        ] {
            assert!(emit_root(src).is_err(), "should reject {src:?}");
        }
        assert!(emit_root(&"a".repeat(4097)).is_err());
        assert!(emit_root(&"a".repeat(4096)).is_ok());
    }

    #[test]
    fn name_and_ext_hygiene() {
        for n in ["a/b", "a*", "a?", "[a]", "a\\b", ".", ".."] {
            assert!(name_ok(n).is_err(), "{n}");
        }
        assert!(name_ok(".env").is_ok());
        assert!(name_ok("it's a \"file\" $x;`y`.txt").is_ok());
        assert!(ext_ok("c++").is_ok());
        for e in ["", "a.b", "a b", "ä", &"x".repeat(17)] {
            assert!(ext_ok(e).is_err(), "{e}");
        }
        assert_eq!(
            v("find files named \"a/b\"").unwrap_err().code,
            "NameHasSlash"
        );
    }

    #[test]
    fn window_arithmetic_is_exact() {
        let w = v("files modified in the last 7 days")
            .unwrap()
            .window
            .unwrap();
        assert_eq!((w.width_secs, w.cutoff, w.t0), (604_800, T0 - 604_800, T0));
        // equivalent durations
        let secs = |s: &str| v(s).unwrap().window.unwrap().width_secs;
        assert_eq!(secs("files modified in the last 6 days"), 144 * 3600);
        assert_eq!(
            secs("files modified in the last 6 days"),
            secs("files modified in the last 144 hours")
        );
        assert_eq!(
            secs("files modified in the last 6 days"),
            secs("files modified in the last 8640 minutes")
        );
        assert_eq!(secs("files modified in the last 7 days"), 168 * 3600);
        assert_eq!(secs("files modified in the last 10000 minutes"), 600_000);
        assert_eq!(secs("files modified in the last hour"), 3600);
    }

    #[test]
    fn window_before_epoch_declines() {
        let req = "files modified in the last 10000 weeks";
        let Outcome::Accept(f) = parse(req) else {
            panic!()
        };
        // 6.048e9 s > T0
        assert_eq!(verify(&f, req, T0).unwrap_err().code, "WindowOverflow");
        assert_eq!(verify(&f, req, 0).unwrap_err().code, "ClockRange");
    }

    #[test]
    fn source_association_catches_tampered_facts() {
        let req = "find python files in docs";
        let Outcome::Accept(mut f) = parse(req) else {
            panic!()
        };
        f.root.source = "other".into();
        assert_eq!(verify(&f, req, T0).unwrap_err().code, "SourceMismatch");
        let Outcome::Accept(mut f) = parse(req) else {
            panic!()
        };
        f.filter = Some(Filter::ExtWord {
            ext: "rs",
            span: match f.filter.unwrap() {
                Filter::ExtWord { span, .. } => span,
                _ => panic!(),
            },
        });
        assert_eq!(verify(&f, req, T0).unwrap_err().code, "SourceMismatch");
    }
}
