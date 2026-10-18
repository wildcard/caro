//! Append-only JSONL decision log for `caro guard`, plus `caro guard report`.
//!
//! One redacted record per shell decision. Shadow mode's value is this log:
//! it shows what Caro *would* have denied or asked on real agent traffic.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{risk_str, GuardMode, Harness, HookEvent, Outcome, Verdict};
use crate::logging::Redaction;
use once_cell::sync::Lazy;
use regex::Regex;

/// Secret shapes the generic [`Redaction`] pattern misses in shell commands:
/// HTTP auth headers, URL userinfo, `curl -u user:pass`, and env assignments
/// whose *name* says secret (`export GITHUB_TOKEN=...`, `DB_PASSWORD='x y'`).
static COMMAND_SECRETS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    [
        (
            r#"(?i)(authorization\s*:\s*)(?:bearer|basic|token)?\s*[^\s'"]+"#,
            "${1}[REDACTED]",
        ),
        (r#"(?i)\b([a-z][a-z0-9+.-]*://)[^/\s:@'"]+:[^/\s@'"]+@"#, "${1}[REDACTED]@"),
        (
            r#"(\s(?:-u|--user)[\s=]+)(?:'[^':]*:[^']*'|"[^":]*:[^"]*"|[^\s:'"]+:[^\s'"]+)"#,
            "${1}[REDACTED]",
        ),
        (
            r#"(?i)\b([a-z0-9_]*(?:key|token|secret|passw(?:or)?d|pwd|credentials?|auth)[a-z0-9_]*)=(?:'[^']*'|"[^"]*"|[^\s;&|]+)"#,
            "${1}=[REDACTED]",
        ),
        // `--password X` / `--passwd X` (space-separated; `=` is covered above).
        (
            r#"(?i)(\s--passw(?:or)?d\s+)(?:'[^']*'|"[^"]*"|[^\s;&|]+)"#,
            "${1}[REDACTED]",
        ),
        // `-p PASS` only where `-p` means password: registry logins and
        // sshpass. `mkdir -p`, `cp -p`, `docker run -p 80:80` stay untouched.
        (
            r#"(?i)(\b(?:(?:docker|podman|nerdctl|helm|oras|buildah)\s+(?:registry\s+)?login\b[^;&|]*?|sshpass)\s-p\s*)(?:'[^']*'|"[^"]*"|[^\s;&|]+)"#,
            "${1}[REDACTED]",
        ),
        // MySQL-family attached password: `mysql -uroot -pSECRET`.
        (
            r#"(\b(?:mysql(?:dump|admin|import|show|check|slap)?|mariadb(?:-(?:dump|admin|import|show|check|slap))?)\b[^;&|]*?\s-p)(?:'[^']*'|"[^"]*"|[^\s;&|]+)"#,
            "${1}[REDACTED]",
        ),
    ]
    .into_iter()
    .map(|(p, r)| (Regex::new(p).expect("valid redaction regex"), r))
    .collect()
});

/// Redact a command or description before it is written to the log.
pub fn redact_command(text: &str) -> String {
    let mut out = Redaction::redact(text);
    for (re, rep) in COMMAND_SECRETS.iter() {
        out = re.replace_all(&out, *rep).into_owned();
    }
    out
}
use crate::models::RiskLevel;

/// One decision, as written to `decisions.jsonl`.
///
/// Field names are a forward-compatible subset of the jev Phase 2
/// `caro decide` record (`source`, `floor_applied`, `latency_us`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionRecord {
    pub ts: String,
    pub harness: Harness,
    pub mode: GuardMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// The harness tool name (`Bash`, `Read`, …); absent in records written
    /// before spec 011 §P2.3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// `PostToolUse` for a leak record (spec 011 §P2.5); absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// The file-tool path, redacted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Redacted via [`Redaction`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The agent's stated intent, redacted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<RiskLevel>,
    #[serde(default)]
    pub matched_patterns: Vec<String>,
    #[serde(default)]
    pub floor_applied: bool,
    /// What Caro decided.
    pub verdict: Verdict,
    /// What reached the harness (`none` in shadow mode).
    pub emitted: Verdict,
    /// Decision provenance; `static` until an LLM judge exists.
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub latency_us: u64,
    pub caro_version: String,
    /// `hash` of the previous record in the file; empty for the first record
    /// (spec 011 §P2.4). Absent in records written before the chain existed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prev_hash: String,
    /// SHA-256 (hex) of this record's canonical JSON with `hash` empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hash: String,
}

impl DecisionRecord {
    /// Hex SHA-256 over the canonical JSON of `self` with `hash` cleared.
    /// Field order is the struct order, so serialization is deterministic.
    pub fn compute_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut clone = self.clone();
        clone.hash.clear();
        let bytes = serde_json::to_vec(&clone).expect("record serializes");
        let digest = Sha256::digest(&bytes);
        let mut hex = String::with_capacity(64);
        for b in digest {
            use std::fmt::Write as _;
            let _ = write!(hex, "{b:02x}");
        }
        hex
    }
}

impl DecisionRecord {
    pub(super) fn new(
        harness: Harness,
        mode: GuardMode,
        event: &HookEvent,
        outcome: &Outcome,
        emitted: Verdict,
        latency_us: u64,
    ) -> Self {
        let (risk, patterns, floor, verdict, error) = match outcome {
            Outcome::Decided(d) => (
                Some(d.risk),
                d.matched_patterns.clone(),
                d.floor_applied,
                d.verdict,
                None,
            ),
            Outcome::Error(e) => (None, Vec::new(), false, Verdict::Ask, Some(e.clone())),
            Outcome::NotShell => (None, Vec::new(), false, Verdict::None, None),
            // A leak record carries the class labels only, never the text.
            Outcome::Leak(classes) => (
                None,
                classes.iter().map(|c| format!("leak: {c}")).collect(),
                false,
                Verdict::None,
                None,
            ),
        };
        Self {
            ts: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            harness,
            mode,
            session_id: event.session_id.clone(),
            tool_use_id: event.tool_use_id.clone(),
            cwd: event.cwd.clone(),
            tool: event.tool_name.clone(),
            event: matches!(outcome, Outcome::Leak(_)).then(|| "PostToolUse".to_string()),
            path: event.file_path.as_deref().map(redact_command),
            command: event.command.as_deref().map(redact_command),
            description: event.description.as_deref().map(redact_command),
            risk,
            matched_patterns: patterns,
            floor_applied: floor,
            verdict,
            emitted,
            source: "static".into(),
            error,
            latency_us,
            caro_version: env!("CARGO_PKG_VERSION").into(),
            prev_hash: String::new(),
            hash: String::new(),
        }
    }
}

/// `<data_dir>/caro/guard/decisions.jsonl`, or `None` if the platform has no
/// data dir.
pub fn default_log_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("caro").join("guard").join("decisions.jsonl"))
}

/// Append one record as a single line in a single `write` on an `O_APPEND`
/// file. The file is created owner-only (0600): it indexes every command an
/// agent ran.
///
/// The record is chained (spec 011 §P2.4): `prev_hash` is set to the `hash`
/// of the file's last record and `hash` is computed over the result, under
/// an exclusive advisory lock so two harness sessions cannot both chain onto
/// the same predecessor. Returns the record as written.
pub fn append(path: &Path, record: &DecisionRecord) -> std::io::Result<DecisionRecord> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut opts = OpenOptions::new();
    opts.create(true).read(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let f = opts.open(path)?;
    // `mode` only applies on create; tighten a pre-existing log too.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = f.set_permissions(std::fs::Permissions::from_mode(0o600));
    }
    let mut lock = fd_lock::RwLock::new(f);
    let mut guard = lock.write()?;

    let mut chained = record.clone();
    chained.prev_hash = last_hash(&mut guard)?;
    chained.hash = chained.compute_hash();

    let mut line = serde_json::to_vec(&chained).map_err(std::io::Error::other)?;
    line.push(b'\n');
    guard.write_all(&line)?;
    Ok(chained)
}

/// How much of the file's tail is read to find the last record.
const TAIL_BYTES: u64 = 64 * 1024;

/// The `hash` of the last parsable record in the file, or empty when the
/// file is empty or its last record predates the chain.
fn last_hash(f: &mut std::fs::File) -> std::io::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let len = f.metadata()?.len();
    if len == 0 {
        return Ok(String::new());
    }
    let start = len.saturating_sub(TAIL_BYTES);
    f.seek(SeekFrom::Start(start))?;
    let mut tail = Vec::with_capacity((len - start) as usize);
    f.read_to_end(&mut tail)?;
    let text = String::from_utf8_lossy(&tail);
    Ok(text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .and_then(|l| serde_json::from_str::<DecisionRecord>(l).ok())
        .map(|r| r.hash)
        .unwrap_or_default())
}

/// Result of walking the hash chain (`caro guard audit verify`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ChainCheck {
    /// Records that carry a hash and were checked.
    pub chained: usize,
    /// Records written before the chain existed (no hash). They are counted
    /// and skipped; the chain restarts after them.
    pub legacy: usize,
    /// Lines that did not parse as a record (torn writes, garbage).
    pub unparsable: usize,
    /// First break: (1-based line number, reason).
    pub first_break: Option<(usize, String)>,
}

impl ChainCheck {
    pub fn ok(&self) -> bool {
        self.first_break.is_none()
    }

    pub fn format(&self, path: &Path) -> String {
        let mut s = format!(
            "caro guard audit verify: {}\n  chained records: {}\n",
            path.display(),
            self.chained
        );
        if self.legacy > 0 {
            s.push_str(&format!("  legacy (unchained) records: {}\n", self.legacy));
        }
        if self.unparsable > 0 {
            s.push_str(&format!("  unparsable lines: {}\n", self.unparsable));
        }
        match &self.first_break {
            None => s.push_str("status: CHAIN INTACT\n"),
            Some((line, why)) => {
                s.push_str(&format!("status: CHAIN BROKEN at line {line}: {why}\n"));
            }
        }
        s
    }
}

/// Walk a decision log and check that every chained record's `hash` matches
/// its content and its `prev_hash` matches the previous chained record. An
/// unparsable line is counted and reported as a break: a torn write is a
/// gap the chain cannot vouch for.
pub fn verify_chain<R: std::io::BufRead>(reader: R) -> std::io::Result<ChainCheck> {
    let mut check = ChainCheck::default();
    let mut expected_prev = String::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let n = idx + 1;
        let Ok(rec) = serde_json::from_str::<DecisionRecord>(&line) else {
            check.unparsable += 1;
            if check.first_break.is_none() {
                check.first_break = Some((n, "line is not a decision record".into()));
            }
            continue;
        };
        if rec.hash.is_empty() {
            check.legacy += 1;
            expected_prev.clear();
            continue;
        }
        check.chained += 1;
        if check.first_break.is_some() {
            continue;
        }
        if rec.prev_hash != expected_prev {
            check.first_break = Some((n, "prev_hash does not match the previous record".into()));
        } else if rec.compute_hash() != rec.hash {
            check.first_break = Some((n, "hash does not match the record's content".into()));
        }
        expected_prev = rec.hash.clone();
    }
    Ok(check)
}

/// One line per record for `caro guard audit tail`.
pub fn format_tail_line(rec: &DecisionRecord) -> String {
    if rec.event.as_deref() == Some("PostToolUse") {
        return format!(
            "{}  leak  {:<8}  {:<8}  {:<6}  {}",
            rec.ts,
            "output",
            format!("{:?}", rec.harness).to_lowercase(),
            rec.tool
                .as_deref()
                .map(str::to_ascii_lowercase)
                .unwrap_or_else(|| "bash".into()),
            rec.matched_patterns
                .iter()
                .map(|p| p.trim_start_matches("leak: "))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    format!(
        "{}  {:<4}  {:<8}  {:<8}  {:<6}  {}",
        rec.ts,
        rec.verdict.as_str(),
        rec.risk.map(risk_str).unwrap_or("error"),
        format!("{:?}", rec.harness).to_lowercase(),
        rec.tool
            .as_deref()
            .map(str::to_ascii_lowercase)
            .unwrap_or_else(|| "bash".into()),
        rec.command
            .as_deref()
            .or(rec.path.as_deref())
            .map(|c| c.escape_debug().to_string())
            .unwrap_or_else(|| "-".into()),
    )
}

/// Summary of a decision log for `caro guard report`.
#[derive(Debug, Default, PartialEq)]
pub struct Report {
    pub total: usize,
    pub unparsable: usize,
    pub by_verdict: BTreeMap<String, usize>,
    pub by_harness: BTreeMap<String, usize>,
    /// Decisions per tool name (lowercase); `bash` for pre-P2.3 records.
    pub by_tool: BTreeMap<String, usize>,
    /// PostToolUse records whose output matched a credential shape.
    pub leaks: usize,
    pub top_patterns: Vec<(String, usize)>,
    pub p50_latency_us: Option<u64>,
    pub p95_latency_us: Option<u64>,
    /// Most recent would-be deny/ask records, newest first.
    pub recent_flagged: Vec<DecisionRecord>,
}

/// Summarize a JSONL decision log held in memory (tests, small logs).
pub fn summarize(contents: &str, limit: usize) -> Report {
    summarize_lines(contents.lines().map(str::to_owned), limit)
}

/// Summarize a decision log by streaming it line by line, so an append-only
/// log that has grown large is never loaded whole. Unparsable lines are
/// counted, not fatal; an I/O read error stops the scan and is returned.
pub fn summarize_reader<R: std::io::BufRead>(reader: R, limit: usize) -> std::io::Result<Report> {
    // Stop at the first read error: some (e.g. EISDIR) repeat forever.
    let mut err = None;
    let lines = reader.lines().map_while(|l| match l {
        Ok(line) => Some(line),
        Err(e) => {
            err = Some(e);
            None
        }
    });
    let report = summarize_lines(lines, limit);
    match err {
        Some(e) => Err(e),
        None => Ok(report),
    }
}

fn summarize_lines(lines: impl Iterator<Item = String>, limit: usize) -> Report {
    let mut report = Report::default();
    let mut patterns: BTreeMap<String, usize> = BTreeMap::new();
    let mut latencies = Vec::new();
    // Only the newest `limit` flagged records are kept.
    let mut flagged: std::collections::VecDeque<DecisionRecord> =
        std::collections::VecDeque::with_capacity(limit.min(1024));

    for line in lines.filter(|l| !l.trim().is_empty()) {
        let Ok(rec) = serde_json::from_str::<DecisionRecord>(&line) else {
            report.unparsable += 1;
            continue;
        };
        report.total += 1;
        *report
            .by_verdict
            .entry(rec.verdict.as_str().to_string())
            .or_default() += 1;
        *report
            .by_harness
            .entry(format!("{:?}", rec.harness).to_lowercase())
            .or_default() += 1;
        *report
            .by_tool
            .entry(
                rec.tool
                    .as_deref()
                    .map(str::to_ascii_lowercase)
                    .unwrap_or_else(|| "bash".into()),
            )
            .or_default() += 1;
        if rec.event.as_deref() == Some("PostToolUse") {
            report.leaks += 1;
        }
        for p in &rec.matched_patterns {
            *patterns.entry(p.clone()).or_default() += 1;
        }
        latencies.push(rec.latency_us);
        if rec.verdict != Verdict::None && limit > 0 {
            if flagged.len() == limit {
                flagged.pop_front();
            }
            flagged.push_back(rec);
        }
    }

    let mut top: Vec<(String, usize)> = patterns.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top.truncate(10);
    report.top_patterns = top;

    latencies.sort_unstable();
    let pct = |p: usize| {
        latencies
            .get((latencies.len().saturating_sub(1)) * p / 100)
            .copied()
    };
    report.p50_latency_us = pct(50);
    report.p95_latency_us = pct(95);

    report.recent_flagged = flagged.into_iter().rev().collect();
    report
}

/// Human-readable rendering of a [`Report`].
pub fn format_report(path: &Path, r: &Report) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(s, "caro guard report: {}", path.display());
    let _ = writeln!(
        s,
        "  decisions: {}{}",
        r.total,
        if r.unparsable > 0 {
            format!(" ({} unparsable lines skipped)", r.unparsable)
        } else {
            String::new()
        }
    );
    if r.total == 0 {
        return s;
    }
    let join = |m: &BTreeMap<String, usize>| {
        m.iter()
            .map(|(k, v)| format!("{k} {v}"))
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let _ = writeln!(s, "  by verdict: {}", join(&r.by_verdict));
    let _ = writeln!(s, "  by harness: {}", join(&r.by_harness));
    let _ = writeln!(s, "  by tool:    {}", join(&r.by_tool));
    if r.leaks > 0 {
        let _ = writeln!(s, "  output leaks: {} (see top patterns)", r.leaks);
    }
    if let (Some(p50), Some(p95)) = (r.p50_latency_us, r.p95_latency_us) {
        let _ = writeln!(s, "  latency:    p50 {p50}µs · p95 {p95}µs");
    }
    if !r.top_patterns.is_empty() {
        let _ = writeln!(s, "  top patterns:");
        for (p, n) in &r.top_patterns {
            let _ = writeln!(s, "    {n:>5}  {p}");
        }
    }
    if !r.recent_flagged.is_empty() {
        let _ = writeln!(s, "  recent would-be deny/ask (newest first):");
        for rec in &r.recent_flagged {
            let _ = writeln!(
                s,
                "    {}  {:<4}  {:<8}  {:<8}  {}",
                rec.ts,
                rec.verdict.as_str(),
                rec.risk.map(risk_str).unwrap_or("error"),
                format!("{:?}", rec.harness).to_lowercase(),
                // Agent-supplied text: escape newlines and control
                // sequences so it cannot forge report lines or drive the
                // reviewer's terminal.
                rec.command
                    .as_deref()
                    .or(rec.path.as_deref())
                    .map(|c| c.escape_debug().to_string())
                    .unwrap_or_else(|| "-".into()),
            );
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guard::{run, GuardMode, Harness};
    use crate::safety::{SafetyConfig, SafetyValidator};

    #[tokio::test]
    async fn append_then_summarize_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("decisions.jsonl");
        let v = SafetyValidator::new(SafetyConfig::moderate()).unwrap();

        for cmd in ["rm -rf /", "ls -la", "rm -rf / --no-preserve-root"] {
            let raw =
                serde_json::json!({"tool_name":"Bash","tool_input":{"command":cmd}}).to_string();
            let (_, _, rec) = run(
                Ok(&v),
                Harness::Claude,
                GuardMode::Shadow,
                &raw,
                std::time::Instant::now(),
            )
            .await;
            append(&path, &rec.unwrap()).unwrap();
        }
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"garbage\n")
            .unwrap();

        let r = summarize(&std::fs::read_to_string(&path).unwrap(), 10);
        assert_eq!(r.total, 3);
        assert_eq!(r.unparsable, 1);
        assert_eq!(r.by_verdict.get("deny"), Some(&2));
        assert_eq!(r.by_verdict.get("none"), Some(&1));
        assert_eq!(r.by_harness.get("claude"), Some(&3));
        assert_eq!(r.recent_flagged.len(), 2);
        assert!(r.recent_flagged.iter().all(|f| f.emitted == Verdict::None));

        let text = format_report(&path, &r);
        assert!(text.contains("by verdict: deny 2 · none 1"));
    }

    fn record(cmd: &str) -> DecisionRecord {
        let ev = HookEvent {
            tool_name: Some("Bash".into()),
            command: Some(cmd.into()),
            ..Default::default()
        };
        DecisionRecord::new(
            Harness::Claude,
            GuardMode::Shadow,
            &ev,
            &Outcome::Error("x".into()),
            Verdict::None,
            1,
        )
    }

    #[test]
    fn append_chains_records_and_verify_accepts_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("d.jsonl");
        let a = append(&path, &record("a")).unwrap();
        let b = append(&path, &record("b")).unwrap();
        let c = append(&path, &record("c")).unwrap();
        assert_eq!(a.prev_hash, "");
        assert_eq!(b.prev_hash, a.hash);
        assert_eq!(c.prev_hash, b.hash);
        assert_eq!(a.hash.len(), 64);
        assert_eq!(a.compute_hash(), a.hash);

        let check = verify_chain(std::io::Cursor::new(std::fs::read(&path).unwrap())).unwrap();
        assert!(check.ok(), "{check:?}");
        assert_eq!(check.chained, 3);
    }

    #[test]
    fn one_changed_byte_breaks_the_chain_at_that_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("d.jsonl");
        for c in ["a", "b", "c"] {
            append(&path, &record(c)).unwrap();
        }
        let text = std::fs::read_to_string(&path).unwrap();
        // Change the command of record 2 without touching its hash.
        let tampered = text.replacen("\"command\":\"b\"", "\"command\":\"B\"", 1);
        assert_ne!(text, tampered);
        let check = verify_chain(std::io::Cursor::new(tampered)).unwrap();
        assert_eq!(
            check.first_break,
            Some((2, "hash does not match the record's content".to_string()))
        );

        // Deleting record 2 breaks the link at the line that follows it.
        let mut lines: Vec<&str> = text.lines().collect();
        lines.remove(1);
        let check = verify_chain(std::io::Cursor::new(lines.join("\n"))).unwrap();
        assert_eq!(check.first_break.as_ref().map(|b| b.0), Some(2));
        assert!(check.first_break.unwrap().1.contains("prev_hash"));
    }

    #[test]
    fn legacy_unchained_records_are_counted_and_the_chain_restarts_after_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("d.jsonl");
        // A pre-P2.4 record: no hash fields at all.
        let legacy = serde_json::to_string(&record("old")).unwrap();
        std::fs::write(&path, format!("{legacy}\n")).unwrap();
        let a = append(&path, &record("a")).unwrap();
        assert_eq!(a.prev_hash, "", "chain starts fresh after a legacy record");
        let check = verify_chain(std::io::Cursor::new(std::fs::read(&path).unwrap())).unwrap();
        assert!(check.ok(), "{check:?}");
        assert_eq!((check.legacy, check.chained), (1, 1));
    }

    #[test]
    fn torn_line_is_reported_as_a_break() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("d.jsonl");
        append(&path, &record("a")).unwrap();
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"{\"ts\":\"torn")
            .unwrap();
        let check = verify_chain(std::io::Cursor::new(std::fs::read(&path).unwrap())).unwrap();
        assert_eq!(check.unparsable, 1);
        assert_eq!(check.first_break.as_ref().map(|b| b.0), Some(2));
    }

    #[test]
    fn report_streams_and_keeps_only_the_newest_flags() {
        let mk = |cmd: &str, verdict: Verdict| {
            let ev = HookEvent {
                tool_name: Some("Bash".into()),
                command: Some(cmd.into()),
                ..Default::default()
            };
            let mut rec = DecisionRecord::new(
                Harness::Claude,
                GuardMode::Shadow,
                &ev,
                &Outcome::Error("x".into()),
                Verdict::None,
                1,
            );
            rec.verdict = verdict;
            serde_json::to_string(&rec).unwrap()
        };
        let mut log = String::new();
        for i in 0..50 {
            log.push_str(&mk(&format!("cmd-{i}"), Verdict::Ask));
            log.push('\n');
        }
        log.push_str("not json\n");
        let r = summarize_reader(std::io::Cursor::new(log), 3).unwrap();
        assert_eq!(r.total, 50);
        assert_eq!(r.unparsable, 1);
        let cmds: Vec<_> = r
            .recent_flagged
            .iter()
            .map(|f| f.command.clone().unwrap())
            .collect();
        assert_eq!(cmds, ["cmd-49", "cmd-48", "cmd-47"]);
    }

    #[test]
    fn report_stops_on_read_errors() {
        struct Broken;
        impl std::io::Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("EISDIR"))
            }
        }
        let r = summarize_reader(std::io::BufReader::new(Broken), 5);
        assert!(r.is_err());
    }

    #[test]
    fn command_secret_shapes_are_redacted() {
        // The fake secret is assembled at runtime so no credential-shaped
        // literal lives in the source (keeps secret scanners quiet).
        let secret = ["fixture", "Val", "9q7Zt"].concat();
        let templates = [
            "curl -H 'Authorization: Bearer {S}' https://api",
            "curl -H \"authorization: token {S}\" x",
            "psql postgres://admin:{S}@db.internal/app",
            "git clone https://user:{S}@github.com/o/r",
            "curl -u alice:{S} https://h",
            "curl --user=bob:{S} https://h",
            "curl -u 'alice:{S}' https://h",
            "curl --user \"bob:{S}\" https://h",
            "docker login -u me -p {S} registry.example",
            "podman login --username me -p '{S}' quay.io",
            "sshpass -p {S} ssh host",
            "mysql -uroot -p{S} -e 'select 1'",
            "mysqldump -u root -p'{S}' db",
            "mariadb -uroot -p{S} app",
            "mariadb-dump -u root -p{S} app",
            "psql --password {S}",
            "export GITHUB_TOKEN={S}",
            "DB_PASSWORD='{S} words' ./migrate",
            "AWS_SECRET_ACCESS_KEY={S}/x+y= aws s3 ls",
            "OPENAI_API_KEY={S} python app.py",
            "env MY_PWD={S} make deploy",
            "docker login -u me; CREDENTIALS={S} run",
            "mysql --password={S} -e 'select 1'",
            "curl 'https://h/x?api_key={S}'",
            "XAI_API_KEY={S} caro --backend grok 'ls'",
        ];
        for tpl in templates {
            let cmd = tpl.replace("{S}", &secret);
            let red = redact_command(&cmd);
            assert!(!red.contains(&secret), "{cmd:?} -> {red:?}");
        }
        // Ordinary commands are untouched.
        for cmd in [
            "mkdir -p a/b",
            "cp -p src dst",
            "docker run -p 8080:80 nginx",
            "ssh -p 2222 host",
            "ls -la",
            "git push origin main",
            "mkdir -p a/b",
            "cargo test",
        ] {
            assert_eq!(redact_command(cmd), cmd);
        }
    }

    #[cfg(unix)]
    #[test]
    fn log_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("d.jsonl");
        let ev = HookEvent::default();
        let rec = DecisionRecord::new(
            Harness::Generic,
            GuardMode::Shadow,
            &ev,
            &Outcome::Error("x".into()),
            Verdict::None,
            1,
        );
        append(&path, &rec).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        // A pre-existing world-readable log is tightened on the next append.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        append(&path, &rec).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn report_escapes_control_characters_in_commands() {
        let ev = HookEvent {
            tool_name: Some("Bash".into()),
            command: Some("rm -rf /\n    2026-01-01  none  safe  claude  forged\x1b[2J".into()),
            ..Default::default()
        };
        let rec = DecisionRecord::new(
            Harness::Claude,
            GuardMode::Shadow,
            &ev,
            &Outcome::Error("x".into()),
            Verdict::None,
            1,
        );
        let line = serde_json::to_string(&rec).unwrap();
        let text = format_report(Path::new("d.jsonl"), &summarize(&line, 10));
        let flagged: Vec<&str> = text.lines().filter(|l| l.contains("rm -rf")).collect();
        assert_eq!(flagged.len(), 1, "{text}");
        assert!(!text.contains('\x1b'));
        assert!(!text
            .lines()
            .any(|l| l.trim_start().starts_with("2026-01-01")));
    }

    #[test]
    fn command_is_redacted_in_record() {
        let ev = HookEvent {
            tool_name: Some("Bash".into()),
            command: Some("curl -H 'x' https://h?api_key=sk_live_abc123".into()),
            ..Default::default()
        };
        let rec = DecisionRecord::new(
            Harness::Claude,
            GuardMode::Shadow,
            &ev,
            &Outcome::Error("x".into()),
            Verdict::None,
            1,
        );
        assert!(!rec.command.unwrap().contains("sk_live_abc123"));
    }
}
