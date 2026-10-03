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
        (r#"(\s(?:-u|--user)[\s=]+)[^\s:'"]+:[^\s'"]+"#, "${1}[REDACTED]"),
        (
            r#"(?i)\b([a-z0-9_]*(?:key|token|secret|passw(?:or)?d|pwd|credentials?|auth)[a-z0-9_]*)=(?:'[^']*'|"[^"]*"|[^\s;&|]+)"#,
            "${1}=[REDACTED]",
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
        };
        Self {
            ts: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            harness,
            mode,
            session_id: event.session_id.clone(),
            tool_use_id: event.tool_use_id.clone(),
            cwd: event.cwd.clone(),
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
        }
    }
}

/// `<data_dir>/caro/guard/decisions.jsonl`, or `None` if the platform has no
/// data dir.
pub fn default_log_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("caro").join("guard").join("decisions.jsonl"))
}

/// Append one record as a single line in a single `write` on an `O_APPEND`
/// file, so concurrent harness sessions interleave whole lines. The file is
/// created owner-only (0600): it indexes every command an agent ran.
pub fn append(path: &Path, record: &DecisionRecord) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut line = serde_json::to_vec(record).map_err(std::io::Error::other)?;
    line.push(b'\n');
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    f.write_all(&line)
}

/// Summary of a decision log for `caro guard report`.
#[derive(Debug, Default, PartialEq)]
pub struct Report {
    pub total: usize,
    pub unparsable: usize,
    pub by_verdict: BTreeMap<String, usize>,
    pub by_harness: BTreeMap<String, usize>,
    pub top_patterns: Vec<(String, usize)>,
    pub p50_latency_us: Option<u64>,
    pub p95_latency_us: Option<u64>,
    /// Most recent would-be deny/ask records, newest first.
    pub recent_flagged: Vec<DecisionRecord>,
}

/// Summarize a JSONL decision log. Unparsable lines are counted, not fatal.
pub fn summarize(contents: &str, limit: usize) -> Report {
    let mut report = Report::default();
    let mut patterns: BTreeMap<String, usize> = BTreeMap::new();
    let mut latencies = Vec::new();
    let mut flagged = Vec::new();

    for line in contents.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(rec) = serde_json::from_str::<DecisionRecord>(line) else {
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
        for p in &rec.matched_patterns {
            *patterns.entry(p.clone()).or_default() += 1;
        }
        latencies.push(rec.latency_us);
        if rec.verdict != Verdict::None {
            flagged.push(rec);
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

    report.recent_flagged = flagged.into_iter().rev().take(limit).collect();
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
                rec.command.as_deref().unwrap_or("-"),
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
                &v,
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

    #[test]
    fn command_secret_shapes_are_redacted() {
        let cases = [
            (
                "curl -H 'Authorization: Bearer eyJhbGciOi.payload.sig' https://api",
                "eyJhbGciOi",
            ),
            (
                "curl -H \"authorization: token ghp_abc123XYZ\" x",
                "ghp_abc123XYZ",
            ),
            ("psql postgres://admin:hunter2@db.internal/app", "hunter2"),
            ("git clone https://user:s3cr3t@github.com/o/r", "s3cr3t"),
            ("curl -u alice:wonderland https://h", "wonderland"),
            ("curl --user=bob:builder https://h", "builder"),
            ("export GITHUB_TOKEN=ghp_1234567890", "ghp_1234567890"),
            ("DB_PASSWORD='two words' ./migrate", "two words"),
            (
                "AWS_SECRET_ACCESS_KEY=abcd/efgh+ijk= aws s3 ls",
                "abcd/efgh+ijk=",
            ),
            ("OPENAI_API_KEY=sk-proj-xyz python app.py", "sk-proj-xyz"),
            ("env MY_PWD=p4ss make deploy", "p4ss"),
            (
                "docker login -u me -p hunter3; CREDENTIALS=c4fe run",
                "c4fe",
            ),
            ("mysql --password=pa55 -e 'select 1'", "pa55"),
            (
                "curl 'https://h/x?api_key=sk_live_abc123'",
                "sk_live_abc123",
            ),
            ("XAI_API_KEY=xai-abc caro --backend grok 'ls'", "xai-abc"),
        ];
        for (cmd, secret) in cases {
            let red = redact_command(cmd);
            assert!(!red.contains(secret), "{cmd:?} -> {red:?}");
        }
        // Ordinary commands are untouched.
        for cmd in [
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
