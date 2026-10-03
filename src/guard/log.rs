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
            r#"(\bmysql(?:dump|admin|import|show|check|slap)?\b[^;&|]*?\s-p)(?:'[^']*'|"[^"]*"|[^\s;&|]+)"#,
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
    // `mode` only applies on create; tighten a pre-existing log too.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = f.set_permissions(std::fs::Permissions::from_mode(0o600));
    }
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

/// Summarize a JSONL decision log held in memory (tests, small logs).
pub fn summarize(contents: &str, limit: usize) -> Report {
    summarize_lines(contents.lines().map(str::to_owned), limit)
}

/// Summarize a decision log by streaming it line by line, so an append-only
/// log that has grown large is never loaded whole. Unreadable or unparsable
/// lines are counted, not fatal.
pub fn summarize_reader<R: std::io::BufRead>(reader: R, limit: usize) -> Report {
    summarize_lines(
        reader
            .lines()
            .map(|l| l.unwrap_or_else(|_| "\u{0}".to_string())),
        limit,
    )
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
        let r = summarize_reader(std::io::Cursor::new(log), 3);
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
