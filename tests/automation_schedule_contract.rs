//! Contract: `.claude/automation/config/schedule.yaml` is versioned, strictly
//! typed, and every enabled loop points at something that exists.
//!
//! google/ax decodes its specs strictly: an unknown or missing field is an
//! error, not a silent default. Before this test the schedule had no version,
//! one entry with no `enabled` or `timeout_minutes`, and two enabled loops
//! naming skills that don't exist. See
//! `docs/research/2026-09-24-google-ax-lessons.md`.

// Some fields are never read: they exist so strict decoding accepts them.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

const SCHEDULE: &str = ".claude/automation/config/schedule.yaml";
const SUPPORTED_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Schedule {
    version: u32,
    timezone: String,
    technical: BTreeMap<String, Loop>,
    content: BTreeMap<String, Loop>,
    management: BTreeMap<String, Loop>,
    settings: Settings,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Loop {
    description: String,
    schedule: String,
    skill: Option<String>,
    agent: Option<String>,
    prompt: Option<String>,
    enabled: bool,
    timeout_minutes: u32,
    #[serde(default)]
    notify_on_failure: bool,
    #[serde(default)]
    notify_on_completion: bool,
    #[serde(default)]
    notify_on_issues: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    notifications: Notifications,
    retry: Retry,
    logging: Logging,
    features: Features,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Notifications {
    slack_webhook: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Retry {
    max_attempts: u32,
    delay_minutes: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Logging {
    level: String,
    keep_days: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Features {
    dry_run: bool,
    parallel_runs: bool,
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn load() -> Schedule {
    let text = std::fs::read_to_string(root().join(SCHEDULE)).expect("read schedule.yaml");
    serde_yaml::from_str(&text)
        .unwrap_or_else(|e| panic!("{SCHEDULE} does not match the schema: {e}"))
}

fn loops(schedule: &Schedule) -> impl Iterator<Item = (&String, &Loop)> {
    schedule
        .technical
        .iter()
        .chain(&schedule.content)
        .chain(&schedule.management)
}

/// Five cron fields (minute, hour, day of month, month, day of week). Each
/// field is a comma list of `*`, `n` or `a-b`, optionally `/step` with step > 0,
/// and every number must be in the field's range.
fn is_cron(expr: &str) -> bool {
    const RANGES: [(u32, u32); 5] = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 7)];
    let fields: Vec<&str> = expr.split_whitespace().collect();
    fields.len() == 5
        && fields
            .iter()
            .zip(RANGES)
            .all(|(field, (lo, hi))| field.split(',').all(|part| cron_part_ok(part, lo, hi)))
}

fn cron_part_ok(part: &str, lo: u32, hi: u32) -> bool {
    let in_range = |s: &str| s.parse::<u32>().is_ok_and(|n| (lo..=hi).contains(&n));
    let (base, step) = match part.split_once('/') {
        Some((base, step)) => (base, Some(step)),
        None => (part, None),
    };
    if step.is_some_and(|s| !s.parse::<u32>().is_ok_and(|n| n > 0)) {
        return false;
    }
    match base.split_once('-') {
        _ if base == "*" => true,
        Some((a, b)) => {
            in_range(a) && in_range(b) && a.parse::<u32>().ok() <= b.parse::<u32>().ok()
        }
        None => in_range(base),
    }
}

/// `/caro.sync all` → `caro.sync`, resolved as a command or a skill.
fn skill_exists(skill: &str) -> bool {
    let name = skill
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or_default();
    !name.is_empty()
        && (root().join(format!(".claude/commands/{name}.md")).is_file()
            || root()
                .join(format!(".claude/skills/{name}/SKILL.md"))
                .is_file())
}

#[test]
fn schedule_decodes_strictly_with_a_supported_version() {
    let schedule = load();
    assert_eq!(
        schedule.version, SUPPORTED_VERSION,
        "unsupported schedule version; bump SUPPORTED_VERSION only with a migration"
    );
    assert!(!schedule.timezone.is_empty());
    assert!(schedule.settings.retry.max_attempts > 0);
}

#[test]
fn every_loop_is_well_formed() {
    let schedule = load();
    for (name, l) in loops(&schedule) {
        assert!(!l.description.is_empty(), "{name}: empty description");
        assert!(is_cron(&l.schedule), "{name}: bad cron {:?}", l.schedule);
        assert!(l.timeout_minutes > 0, "{name}: timeout_minutes must be > 0");
        assert!(
            l.skill.is_some() != l.agent.is_some(),
            "{name}: set exactly one of `skill` or `agent`"
        );
        assert!(
            l.agent.is_none() || l.prompt.is_some(),
            "{name}: an `agent` loop needs a `prompt`"
        );
    }
}

#[test]
fn enabled_loops_point_at_something_that_exists() {
    let schedule = load();
    for (name, l) in loops(&schedule).filter(|(_, l)| l.enabled) {
        if let Some(skill) = &l.skill {
            assert!(
                skill_exists(skill),
                "{name}: enabled, but skill {skill:?} has no .claude/commands or .claude/skills entry"
            );
        }
        if let Some(agent) = &l.agent {
            assert!(
                root().join(format!(".claude/agents/{agent}.md")).is_file(),
                "{name}: enabled, but agent {agent:?} has no .claude/agents entry"
            );
        }
    }
}

#[test]
fn strict_decoding_rejects_unknown_and_missing_fields() {
    let base = "version: 1\ntimezone: UTC\ncontent: {}\nmanagement: {}\n\
        settings: {notifications: {slack_webhook: null, email: null}, \
        retry: {max_attempts: 1, delay_minutes: 1}, logging: {level: info, keep_days: 1}, \
        features: {dry_run: false, parallel_runs: false}}\n";
    let ok = "technical: {a: {description: d, schedule: '0 0 * * *', skill: /x, enabled: true, timeout_minutes: 5}}\n";
    assert!(serde_yaml::from_str::<Schedule>(&format!("{base}{ok}")).is_ok());

    let missing = "technical: {a: {description: d, schedule: '0 0 * * *', skill: /x}}\n";
    assert!(serde_yaml::from_str::<Schedule>(&format!("{base}{missing}")).is_err());

    let unknown = "technical: {a: {description: d, schedule: '0 0 * * *', skill: /x, enabled: true, timeout_minutes: 5, not_a_field: 1}}\n";
    assert!(serde_yaml::from_str::<Schedule>(&format!("{base}{unknown}")).is_err());
}

#[test]
fn cron_check_rejects_malformed_expressions() {
    assert!(is_cron("0 */4 * * 1-5"));
    assert!(is_cron("5 0,12 1-15/2 * 0"));
    assert!(!is_cron("0 5 * *"));
    assert!(!is_cron("daily"));
    assert!(!is_cron("99 99 * * *"), "out of range");
    assert!(!is_cron("- - - - -"), "empty range ends");
    assert!(!is_cron("*/0 * * * *"), "zero step");
    assert!(!is_cron("0 5-3 * * *"), "reversed range");
    assert!(!is_cron("0 0 0 * *"), "day of month starts at 1");
}
